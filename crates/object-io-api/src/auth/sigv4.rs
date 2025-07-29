//! AWS Signature Version 4 implementation

use axum::http::{HeaderMap, Method};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use object_io_core::ObjectIOError;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;
type Result<T> = std::result::Result<T, ObjectIOError>;

/// AWS SigV4 authentication context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
    pub region: String,
    pub service: String,
}

/// Parsed authorization header
#[derive(Debug, Clone)]
pub struct AuthorizationHeader {
    pub algorithm: String,
    pub credential: String,
    pub signed_headers: Vec<String>,
    pub signature: String,
}

/// SigV4 request for signature validation
#[derive(Debug)]
pub struct SignatureRequest<'a> {
    pub method: &'a Method,
    pub uri: &'a str,
    pub query_string: &'a str,
    pub headers: &'a HeaderMap,
    pub payload_hash: &'a str,
    pub timestamp: DateTime<Utc>,
}

impl AuthorizationHeader {
    /// Parse Authorization header value
    pub fn parse(auth_header: &str) -> Result<Self> {
        if !auth_header.starts_with("AWS4-HMAC-SHA256 ") {
            return Err(ObjectIOError::AuthError {
                message: "Invalid authorization algorithm".to_string(),
            });
        }

        let auth_parts = auth_header
            .strip_prefix("AWS4-HMAC-SHA256 ")
            .unwrap_or("")
            .split(", ")
            .collect::<Vec<_>>();

        let mut credential = None;
        let mut signed_headers = None;
        let mut signature = None;

        for part in auth_parts {
            if let Some(cred) = part.strip_prefix("Credential=") {
                credential = Some(cred.to_string());
            } else if let Some(headers) = part.strip_prefix("SignedHeaders=") {
                signed_headers = Some(
                    headers
                        .split(';')
                        .map(|h| h.to_string())
                        .collect::<Vec<_>>()
                );
            } else if let Some(sig) = part.strip_prefix("Signature=") {
                signature = Some(sig.to_string());
            }
        }

        Ok(AuthorizationHeader {
            algorithm: "AWS4-HMAC-SHA256".to_string(),
            credential: credential.ok_or_else(|| ObjectIOError::AuthError {
                message: "Missing credential in authorization header".to_string(),
            })?,
            signed_headers: signed_headers.ok_or_else(|| ObjectIOError::AuthError {
                message: "Missing signed headers in authorization header".to_string(),
            })?,
            signature: signature.ok_or_else(|| ObjectIOError::AuthError {
                message: "Missing signature in authorization header".to_string(),
            })?,
        })
    }

    /// Extract access key from credential
    pub fn access_key(&self) -> Result<String> {
        let parts: Vec<&str> = self.credential.split('/').collect();
        if parts.is_empty() {
            return Err(ObjectIOError::AuthError {
                message: "Invalid credential format".to_string(),
            });
        }
        Ok(parts[0].to_string())
    }
}

/// SigV4 signature validator
pub struct SigV4Validator {
    region: String,
    service: String,
}

impl SigV4Validator {
    /// Create new SigV4 validator
    pub fn new(region: String, service: String) -> Self {
        Self { region, service }
    }

    /// Validate SigV4 signature
    pub fn validate_signature(
        &self,
        request: &SignatureRequest,
        auth_header: &AuthorizationHeader,
        secret_key: &str,
    ) -> Result<bool> {
        // Generate expected signature
        let expected_signature = self.generate_signature(request, secret_key)?;
        
        // Compare signatures (constant-time comparison)
        let expected_bytes = hex::decode(&expected_signature).map_err(|_| {
            ObjectIOError::AuthError {
                message: "Failed to decode expected signature".to_string(),
            }
        })?;
        
        let provided_bytes = hex::decode(&auth_header.signature).map_err(|_| {
            ObjectIOError::AuthError {
                message: "Failed to decode provided signature".to_string(),
            }
        })?;

        Ok(constant_time_eq(&expected_bytes, &provided_bytes))
    }

    /// Generate SigV4 signature
    fn generate_signature(&self, request: &SignatureRequest, secret_key: &str) -> Result<String> {
        // Step 1: Create canonical request
        let canonical_request = self.create_canonical_request(request)?;
        
        // Step 2: Create string to sign
        let string_to_sign = self.create_string_to_sign(&canonical_request, request.timestamp)?;
        
        // Step 3: Calculate signature
        let signing_key = self.derive_signing_key(secret_key, request.timestamp)?;
        let signature = self.calculate_signature(&string_to_sign, &signing_key)?;
        
        Ok(hex::encode(signature))
    }

    /// Create canonical request string
    fn create_canonical_request(&self, request: &SignatureRequest) -> Result<String> {
        let canonical_method = request.method.as_str();
        let canonical_uri = self.canonical_uri(request.uri);
        let canonical_query_string = self.canonical_query_string(request.query_string);
        let canonical_headers = self.canonical_headers(request.headers)?;
        let signed_headers = self.signed_headers(request.headers);

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            canonical_method,
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            request.payload_hash
        );

        Ok(canonical_request)
    }

    /// Create string to sign
    fn create_string_to_sign(&self, canonical_request: &str, timestamp: DateTime<Utc>) -> Result<String> {
        let algorithm = "AWS4-HMAC-SHA256";
        let timestamp_str = timestamp.format("%Y%m%dT%H%M%SZ").to_string();
        let credential_scope = format!(
            "{}/{}/{}/aws4_request",
            timestamp.format("%Y%m%d"),
            self.region,
            self.service
        );
        
        let hashed_canonical_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp_str, credential_scope, hashed_canonical_request
        );

        Ok(string_to_sign)
    }

    /// Derive signing key
    fn derive_signing_key(&self, secret_key: &str, timestamp: DateTime<Utc>) -> Result<Vec<u8>> {
        let date_key = hmac_sha256(
            format!("AWS4{}", secret_key).as_bytes(),
            timestamp.format("%Y%m%d").to_string().as_bytes(),
        )?;
        
        let date_region_key = hmac_sha256(&date_key, self.region.as_bytes())?;
        let date_region_service_key = hmac_sha256(&date_region_key, self.service.as_bytes())?;
        let signing_key = hmac_sha256(&date_region_service_key, b"aws4_request")?;

        Ok(signing_key)
    }

    /// Calculate final signature
    fn calculate_signature(&self, string_to_sign: &str, signing_key: &[u8]) -> Result<Vec<u8>> {
        hmac_sha256(signing_key, string_to_sign.as_bytes())
    }

    /// Create canonical URI
    fn canonical_uri(&self, uri: &str) -> String {
        if uri.is_empty() {
            "/".to_string()
        } else {
            uri.to_string()
        }
    }

    /// Create canonical query string
    fn canonical_query_string(&self, query_string: &str) -> String {
        if query_string.is_empty() {
            return String::new();
        }

        let mut params: Vec<(String, String)> = query_string
            .split('&')
            .filter_map(|param| {
                let parts: Vec<&str> = param.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((
                        percent_encode(parts[0]),
                        percent_encode(parts[1]),
                    ))
                } else {
                    Some((percent_encode(parts[0]), String::new()))
                }
            })
            .collect();

        params.sort_by(|a, b| a.0.cmp(&b.0));

        params
            .into_iter()
            .map(|(key, value)| {
                if value.is_empty() {
                    key
                } else {
                    format!("{}={}", key, value)
                }
            })
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Create canonical headers
    fn canonical_headers(&self, headers: &HeaderMap) -> Result<String> {
        let mut canonical_headers: Vec<(String, String)> = Vec::new();

        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            let header_value = value.to_str().map_err(|_| {
                ObjectIOError::AuthError {
                    message: format!("Invalid header value for {}", header_name),
                }
            })?;
            
            canonical_headers.push((header_name, header_value.trim().to_string()));
        }

        canonical_headers.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(canonical_headers
            .into_iter()
            .map(|(name, value)| format!("{}:{}", name, value))
            .collect::<Vec<_>>()
            .join("\n") + "\n")
    }

    /// Create signed headers
    fn signed_headers(&self, headers: &HeaderMap) -> String {
        let mut header_names: Vec<String> = headers
            .keys()
            .map(|name| name.as_str().to_lowercase())
            .collect();
        
        header_names.sort();
        header_names.join(";")
    }
}

/// HMAC-SHA256 helper function
fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| {
        ObjectIOError::AuthError {
            message: "Invalid HMAC key".to_string(),
        }
    })?;
    
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Simple percent encoding for URL components
fn percent_encode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a[i] ^ b[i];
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, Method};
    use chrono::DateTime;

    #[test]
    fn test_authorization_header_parsing() {
        let auth_header = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20230101/us-east-1/s3/aws4_request, SignedHeaders=host;range;x-amz-date, Signature=fe5f80f77d5fa3beca038a248ff027d0445342fe2855ddc963176630326f1024";
        
        let parsed = AuthorizationHeader::parse(auth_header).unwrap();
        assert_eq!(parsed.algorithm, "AWS4-HMAC-SHA256");
        assert_eq!(parsed.access_key().unwrap(), "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(parsed.signed_headers, vec!["host", "range", "x-amz-date"]);
    }

    #[test]
    fn test_canonical_query_string() {
        let validator = SigV4Validator::new("us-east-1".to_string(), "s3".to_string());
        
        let result = validator.canonical_query_string("prefix=somePrefix&delimiter=%2F&max-keys=2");
        assert!(result.contains("delimiter=%252F"));
        assert!(result.contains("max-keys=2"));
        assert!(result.contains("prefix=somePrefix"));
    }
}
