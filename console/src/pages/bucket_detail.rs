use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::types::ObjectInfo;

#[component]
pub fn BucketDetailPage() -> impl IntoView {
    let bucket_name = "example-bucket".to_string(); // Temporary fixed value
    
    let (objects, set_objects) = signal(Vec::<ObjectInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Load objects on mount
    {
        let bucket_name = bucket_name.clone();
        spawn_local(async move {
            set_loading.set(true);
            match api::list_objects(&bucket_name).await {
                Ok(object_list) => {
                    set_objects.set(object_list);
                    set_error.set(None);
                }
                Err(err) => {
                    set_error.set(Some(err));
                }
            }
            set_loading.set(false);
        });
    }

    view! {
        <div class="buckets-container">
            <div class="buckets-header">
                <h2 class="section-title">
                    "Bucket: " {bucket_name.clone()}
                </h2>
                <div style="display: flex; gap: 1rem;">
                    <button class="btn">
                        "⬆️ Upload Object"
                    </button>
                    <a href="/buckets" class="btn btn-secondary">
                        "← Back to Buckets"
                    </a>
                </div>
            </div>
            
            {move || {
                if loading.get() {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Loading objects..."
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="error">
                            "Failed to load objects: " {err}
                        </div>
                    }.into_any()
                } else {
                    let object_list = objects.get();
                    if object_list.is_empty() {
                        view! {
                            <div style="text-align: center; padding: 2rem;">
                                <p style="color: #6b7280; margin-bottom: 1rem;">
                                    "This bucket is empty. Upload your first object to get started."
                                </p>
                                <button class="btn">
                                    "Upload First Object"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ObjectList objects=object_list/>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}

#[component]
fn ObjectList(objects: Vec<ObjectInfo>) -> impl IntoView {
    let format_size = |bytes: u64| {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    view! {
        <div class="bucket-list">
            {objects.into_iter().map(|object| {
                view! {
                    <div class="bucket-item">
                        <div class="bucket-info">
                            <div class="bucket-name">{object.key.clone()}</div>
                            <div class="bucket-meta">
                                {format!("{} • {} • Modified {}", 
                                    format_size(object.size),
                                    object.content_type,
                                    object.last_modified.format("%Y-%m-%d %H:%M")
                                )}
                            </div>
                        </div>
                        <div class="bucket-actions">
                            <button class="btn btn-small">
                                "Download"
                            </button>
                            <button class="btn btn-small btn-secondary">
                                "Delete"
                            </button>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
