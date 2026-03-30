use crate::ui::{DraggableElement, Styles, active_or};
use dioxus::prelude::*;
use dioxus_html::input_data::MouseButton;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

/// Misc app stats
#[derive(Default)]
pub struct AppStats {
    pub memory_mb: Option<u64>,
    pub virtual_memory_mb: Option<u64>,
    /// percentage of total cpu usage divided among all cores on machine
    pub total_cpu_usage_percentage: Option<f32>,
}

/// Debug menu with misc tools
#[component]
pub fn DebugMenu(active: Signal<bool>) -> Element {
    let mut app_stats = use_signal(|| AppStats::default());

    let mut sys = use_signal(|| System::new_all());

    let cores = use_signal(|| std::thread::available_parallelism().map(|n| n.get() as f32));

    let mut show_controls = use_signal(|| false);

    use_future(move || async move {
        let pid = sysinfo::get_current_pid();

        loop {
            sys.write().refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_cpu(),
            );
            let r = sys.read();

            let Ok(pid) = pid.inspect_err(|err| {
                println!(
                    "Could not get PID. App stats will not update. Error: {}",
                    err
                )
            }) else {
                break;
            };

            let Some(process) = r.process(pid) else {
                println!("Could not get process from PID. App stats will not update.");
                break;
            };

            let r = cores.read();
            let Ok(cores) = r.as_ref().inspect_err(|err| {
                println!(
                    "could not get cpu cores. App stats will not update. Error: {}",
                    err
                )
            }) else {
                break;
            };

            *app_stats.write() = AppStats {
                memory_mb: Some(process.memory() / (1024 * 1024)),
                virtual_memory_mb: Some(process.virtual_memory() / (1024 * 1024)),
                total_cpu_usage_percentage: Some(process.cpu_usage() / cores),
            };
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let mut opacity_slider = use_signal(|| 100);
    let opacity = use_memo(move || {
        let opacity = (*opacity_slider.read() as f32) / 100.0;

        opacity
    });

    let oninput_opacity = move |evt: Event<FormData>| {
        if let Ok(new_val) = evt.value().parse::<i32>() {
            opacity_slider.set(new_val);
        }
    };

    let onmousedown = move |evt: Event<MouseData>| {
        if let Some(button) = evt.data().trigger_button()
            && let MouseButton::Primary = button
        {
            *show_controls.write() ^= true;
        };
    };

    let cpu_usage_str = use_memo(move || {
        app_stats
            .read()
            .total_cpu_usage_percentage
            .map(|n| format!("{:.2}", n))
            .unwrap_or("???".into())
    });

    rsx! {

        DraggableElement {
            div {
                hidden: !*active.read(),
                class: Styles::card,
                background_color: format!("rgba(42, 42, 42, {})", opacity.read()),
                width: "300px",
                height: "300px",
                style: "user-select: none; -webkit-user-select: none; -moz-user-select: none;",
                position: "relative",
                padding_top: "40px",
                button {
                    position: "absolute",
                    top: "8px",
                    left: "8px",
                    padding: "4px 8px",
                    font_size: "12px",
                    background: "rgba(255, 255, 255, 0.2)",
                    class: active_or(*show_controls.read(), Styles::selected, ""),
                    onmousedown,
                    "Controls"

                }
                div {
                    hidden: !*show_controls.read(),
                    style: "
                        position: absolute;
                        top: 40px;
                        left: 8px;
                        background: #333;
                        color: white;
                        border: 1px solid #555;
                        border-radius: 4px;
                        padding: 8px 12px;
                        font-size: 12px;
                        white-space: nowrap;
                        z-index: 20;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.3);
                    ",
                    u {
                        "move menu"
                    }
                    {
                        ": "
                    }
                    "drag with left and right mouse button held on menu"
                }
                button {
                    style: "
                        position: absolute;
                        top: 8px;
                        right: 8px;
                        width: 24px;
                        height: 24px;
                        border: none;
                        background: rgba(255, 255, 255, 0.2);
                        color: white;
                        border-radius: 4px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-weight: bold;
                        font-size: 14px;
                        line-height: 1;
                        z-index: 10;
                    ",
                    onclick: move |_| {
                        active.set(false)
                    },
                    "✕"
                }
                h4 { "opacity: {opacity}" }
                input {
                    r#type: "range",
                    min: "0",
                    max: "100",
                    value: "{opacity_slider}",
                    oninput: oninput_opacity,
                    style: "--fill: {opacity_slider}%"
                }

                div {
                    h1 {
                        "app stats"
                    }
                }

                h3 {
                    {format!("memory: {:#} mb", app_stats.read().memory_mb.map(|n| n.to_string()).unwrap_or("???".into()))}
                }
                h3 {
                    {format!("virtual memory: {:#} mb", app_stats.read().virtual_memory_mb.map(|n| n.to_string()).unwrap_or("???".into()))}
                }
                h3 {
                    {format!("cpu usage: {:#}%", cpu_usage_str)}
                }
            }
        }
    }
}
