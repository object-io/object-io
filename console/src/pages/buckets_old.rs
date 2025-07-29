use leptos::prelude::*;
use leptos_dom::logging::console_log;
use crate::api;
use crate::types::Bucket;
use crate::components::CreateBucketModal;

#[component]
pub fn BucketsPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (buckets, set_buckets) = signal(Vec::<Bucket>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Load buckets on mount
    spawn_local(async move {
        set_loading.set(true);
        match api::list_buckets().await {
            Ok(bucket_list) => {
                set_buckets.set(bucket_list);
                set_error.set(None);
            }
            Err(err) => {
                set_error.set(Some(err));
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="buckets-container">
            <div class="buckets-header">
                <h2 class="section-title">"My Buckets"</h2>
                <button 
                    class="btn"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "🪣 Create Bucket"
                </button>
            </div>

            {move || {
                if loading.get() {
                    view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            "Loading buckets..."
                        </div>
                    }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <div class="error">
                            "Error loading buckets: " {err}
                        </div>
                    }.into_any()
                } else {
                    let bucket_list = buckets.get();
                    if bucket_list.is_empty() {
                        view! {
                            <div style="text-align: center; padding: 2rem;">
                                <p style="color: #6b7280; margin-bottom: 1rem;">
                                    "No buckets found. Create your first bucket to get started."
                                </p>
                                <button 
                                    class="btn"
                                    on:click=move |_| set_show_create_modal.set(true)
                                >
                                    "Create First Bucket"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <BucketList buckets=bucket_list/>
                        }.into_any()
                    }
                }
            }}

            <CreateBucketModal 
                show=show_create_modal
                on_close=move || set_show_create_modal.set(false)
                on_success=move |_bucket_name| {
                    set_show_create_modal.set(false);
                    // Reload buckets
                    spawn_local(async move {
                        if let Ok(bucket_list) = api::list_buckets().await {
                            set_buckets.set(bucket_list);
                        }
                    });
                }
            />
        </div>
    }#[component]
fn BucketList(buckets: Vec<Bucket>) -> impl IntoView {
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
            {buckets.into_iter().map(|bucket| {
                let bucket_name = bucket.name.clone();
                let delete_bucket_name = bucket.name.clone();
                
                view! {
                    <div class="bucket-item">
                        <div class="bucket-info">
                            <div class="bucket-name">{bucket.name.clone()}</div>
                            <div class="bucket-meta">
                                {format!("{} objects • {} • Created {}", 
                                    bucket.objects_count,
                                    format_size(bucket.size_bytes),
                                    bucket.created_at.format("%Y-%m-%d")
                                )}
                            </div>
                        </div>
                        <div class="bucket-actions">
                            <a 
                                href=format!("/buckets/{}", bucket_name)
                                class="btn btn-small"
                            >
                                "View"
                            </a>
                            <button 
                                class="btn btn-small btn-secondary"
                                on:click={
                                    let delete_bucket_name = delete_bucket_name.clone();
                                    move |_| {
                                        let delete_bucket_name = delete_bucket_name.clone();
                                        if web_sys::window()
                                            .unwrap()
                                            .confirm_with_message(&format!("Delete bucket '{}'?", delete_bucket_name))
                                            .unwrap_or(false)
                                        {
                                            spawn_local(async move {
                                                let _ = api::delete_bucket(&delete_bucket_name).await;
                                                // TODO: Refresh the list
                                            });
                                        }
                                    }
                                }
                            >
                                "Delete"
                            </button>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
