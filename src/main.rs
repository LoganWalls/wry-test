use anyhow::Result;
use axum::{routing::get, Router};
use mlua::Lua;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::Mutex;
use wry::WebViewBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let lua = Box::leak(Box::new(Mutex::new(Lua::new())));
    // Start the backend server
    tokio::spawn(async {
        // build our application with a single route
        let app = Router::new().route(
            "/",
            get(|| async {
                lua.lock()
                    .await
                    .load(r#""Hello from LuaJit!""#)
                    .eval::<String>()
                    .unwrap()
            }),
        );
        // run it with hyper on localhost:3000
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
    });

    // Create the display window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let builder = WebViewBuilder::new(&window);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let builder = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    let _webview = builder.with_url("http://localhost:3000")?.build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit
        }
    });
}
