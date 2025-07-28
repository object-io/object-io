use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="buckets-container">
            <div class="buckets-header">
                <h2 class="section-title">"Settings"</h2>
            </div>
            
            <div class="dashboard-grid">
                <div class="dashboard-card">
                    <div class="card-header">
                        <h3 class="card-title">"Server Configuration"</h3>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 1rem;">
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Storage Backend"
                            </label>
                            <select style="width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 0.375rem;">
                                <option>"Filesystem"</option>
                                <option>"AWS S3"</option>
                                <option>"Google Cloud Storage"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Default Region"
                            </label>
                            <input 
                                type="text"
                                value="us-east-1"
                                style="width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 0.375rem;"
                            />
                        </div>
                        <button class="btn">
                            "Save Settings"
                        </button>
                    </div>
                </div>
                
                <div class="dashboard-card">
                    <div class="card-header">
                        <h3 class="card-title">"Security"</h3>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 1rem;">
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Access Key Management"
                            </label>
                            <button class="btn btn-secondary">
                                "🔑 Manage API Keys"
                            </button>
                        </div>
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Bucket Policies"
                            </label>
                            <button class="btn btn-secondary">
                                "📋 Configure Policies"
                            </button>
                        </div>
                    </div>
                </div>
                
                <div class="dashboard-card">
                    <div class="card-header">
                        <h3 class="card-title">"Monitoring"</h3>
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 1rem;">
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Logging Level"
                            </label>
                            <select style="width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 0.375rem;">
                                <option>"ERROR"</option>
                                <option>"WARN"</option>
                                <option selected>"INFO"</option>
                                <option>"DEBUG"</option>
                                <option>"TRACE"</option>
                            </select>
                        </div>
                        <div>
                            <label style="display: block; margin-bottom: 0.5rem; font-weight: 500;">
                                "Metrics Export"
                            </label>
                            <button class="btn btn-secondary">
                                "📊 Configure Metrics"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
