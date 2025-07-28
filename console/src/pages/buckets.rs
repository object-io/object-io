use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api;
use crate::types::{Bucket, CreateBucketRequest};
use crate::components::CreateBucketModal;

#[component]
pub fn BucketsPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let buckets = leptos::prelude::Resource::new(|| (), |_| api::list_buckets());

    view! {
        <div class="buckets-container">
            <div class="buckets-header">
                <h2 class="section-title">"Buckets"</h2>
                <button 
                    class="btn"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    "➕ Create Bucket"
                </button>
            </div>
            
            <Suspense fallback=move || view! { <div class="loading"><div class="spinner"></div>"Loading buckets..."</div> }>
                {move || {
                    buckets.get().map(|result| {
                        match result {
                            Ok(buckets) => {
                                if buckets.is_empty() {
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
                                    }.into_view()
                                } else {
                                    view! {
                                        <BucketList buckets=buckets/>
                                    }.into_view()
                                }
                            },
                            Err(err) => view! {
                                <div class="error">
                                    "Failed to load buckets: " {err}
                                </div>
                            }.into_view(),
                        }
                    })
                }}
            </Suspense>
        </div>
        
        <CreateBucketModal 
            show=show_create_modal
            on_close=move || set_show_create_modal.set(false)
            on_success=move |_| {
                set_show_create_modal.set(false);
                buckets.refetch();
            }
        />
    }
}

#[component]
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
                                on:click=move |_| {
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
