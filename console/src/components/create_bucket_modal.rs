use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api;
use crate::types::CreateBucketRequest;

#[component]
pub fn CreateBucketModal<F>(
    show: ReadSignal<bool>,
    on_close: F,
    on_success: impl Fn(String) + 'static + Clone,
) -> impl IntoView 
where
    F: Fn() + 'static + Clone,
{
    let (bucket_name, set_bucket_name) = create_signal(String::new());
    let (region, set_region) = create_signal("us-east-1".to_string());
    let (creating, set_creating) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);

    let on_close_clone = on_close.clone();
    let close_modal = move || {
        set_bucket_name.set(String::new());
        set_region.set("us-east-1".to_string());
        set_creating.set(false);
        set_error.set(None);
        on_close_clone();
    };

    let create_bucket = move |_| {
        let name = bucket_name.get();
        if name.trim().is_empty() {
            set_error.set(Some("Bucket name is required".to_string()));
            return;
        }

        set_creating.set(true);
        set_error.set(None);
        
        let request = CreateBucketRequest {
            name: name.clone(),
            region: region.get(),
        };
        
        let on_success_clone = on_success.clone();
        spawn_local(async move {
            match api::create_bucket(request).await {
                Ok(_) => {
                    on_success_clone(name);
                    close_modal();
                },
                Err(err) => {
                    set_error.set(Some(err));
                    set_creating.set(false);
                }
            }
        });
    };

    view! {
        <div 
            class="modal-overlay"
            style=move || if show.get() { 
                "display: flex; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); z-index: 1000; align-items: center; justify-content: center;"
            } else { 
                "display: none;" 
            }
            on:click=move |e| {
                if e.target() == e.current_target() {
                    close_modal();
                }
            }
        >
            <div 
                class="modal-content"
                style="background: white; border-radius: 1rem; padding: 2rem; width: 90%; max-width: 500px; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);"
            >
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;">
                    <h3 style="font-size: 1.25rem; font-weight: 600; color: #1f2937;">
                        "Create New Bucket"
                    </h3>
                    <button 
                        style="background: none; border: none; font-size: 1.5rem; cursor: pointer; color: #6b7280;"
                        on:click=move |_| close_modal()
                    >
                        "×"
                    </button>
                </div>
                
                {move || {
                    error.get().map(|err| view! {
                        <div class="error" style="margin-bottom: 1rem;">
                            {err}
                        </div>
                    })
                }}
                
                <div style="display: flex; flex-direction: column; gap: 1rem; margin-bottom: 1.5rem;">
                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500; color: #374151;">
                            "Bucket Name"
                        </label>
                        <input 
                            type="text"
                            placeholder="my-awesome-bucket"
                            style="width: 100%; padding: 0.75rem; border: 1px solid #d1d5db; border-radius: 0.375rem; font-size: 1rem;"
                            prop:value=bucket_name
                            on:input=move |e| set_bucket_name.set(event_target_value(&e))
                        />
                        <p style="font-size: 0.875rem; color: #6b7280; margin-top: 0.25rem;">
                            "Bucket names must be globally unique and follow DNS naming conventions"
                        </p>
                    </div>
                    
                    <div>
                        <label style="display: block; margin-bottom: 0.5rem; font-weight: 500; color: #374151;">
                            "Region"
                        </label>
                        <select 
                            style="width: 100%; padding: 0.75rem; border: 1px solid #d1d5db; border-radius: 0.375rem; font-size: 1rem;"
                            prop:value=region
                            on:change=move |e| set_region.set(event_target_value(&e))
                        >
                            <option value="us-east-1">"US East (N. Virginia)"</option>
                            <option value="us-west-2">"US West (Oregon)"</option>
                            <option value="eu-west-1">"Europe (Ireland)"</option>
                            <option value="ap-southeast-1">"Asia Pacific (Singapore)"</option>
                        </select>
                    </div>
                </div>
                
                <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                    <button 
                        class="btn btn-secondary"
                        disabled=creating
                        on:click=move |_| close_modal()
                    >
                        "Cancel"
                    </button>
                    <button 
                        class="btn"
                        disabled=creating
                        on:click=create_bucket
                    >
                        {move || if creating.get() { "Creating..." } else { "Create Bucket" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
