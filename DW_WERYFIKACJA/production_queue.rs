use photocraft_ui_egui::{PhotocraftApp, Services, menus, export_dialog::batch};
use serde_json::json;
use std::{path::PathBuf, time::{Duration,Instant}};

fn snapshot(app:&mut PhotocraftApp, root:&std::path::Path,w:u32,h:u32) {
    use egui_wgpu::wgpu;
    let instance=wgpu::Instance::new(wgpu::InstanceDescriptor {backends:wgpu::Backends::VULKAN,..wgpu::InstanceDescriptor::new_without_display_handle()});
    let adapter=pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {force_fallback_adapter:true,..Default::default()})).unwrap();
    let (device,queue)=pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format=wgpu::TextureFormat::Rgba8Unorm;
    let mut renderer=egui_wgpu::Renderer::new(&device,format,egui_wgpu::RendererOptions::PREDICTABLE);
    let mut capture=egui_wgpu::capture::CaptureState::new(&device,wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},format);
    let screen=egui_wgpu::ScreenDescriptor {size_in_pixels:[w,h],pixels_per_point:1.0};
    let ctx=egui::Context::default();PhotocraftApp::setup_context(&ctx,Default::default());
    let mut final_output=None;
    for frame in 0..12 {
        let out=ctx.run_ui(egui::RawInput {time:Some(frame as f64/30.0),screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(w as f32,h as f32))), ..Default::default()},|ui|{batch::show(app,ui.ctx());});
        for (id,deltas) in &out.textures_delta.set {for delta in deltas {renderer.update_texture(&device,&queue,*id,delta);}}
        final_output=Some(out);
    }
    let out=final_output.unwrap();
    let jobs=ctx.tessellate(out.shapes,out.pixels_per_point);
    let mut encoder=device.create_command_encoder(&Default::default());
    let mut commands=renderer.update_buffers(&device,&queue,&mut encoder,&jobs,&screen);
    {
        let view=capture.texture.create_view(&Default::default());
        let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments:&[Some(wgpu::RenderPassColorAttachment {view:&view,resolve_target:None,depth_slice:None,ops:wgpu::Operations {load:wgpu::LoadOp::Clear(wgpu::Color::WHITE),store:wgpu::StoreOp::Store}})],..Default::default()
        }).forget_lifetime();
        renderer.render(&mut pass,&jobs,&screen);
    }
    let buffer=capture.copy_textures(&device,None,&mut encoder);
    commands.push(encoder.finish());queue.submit(commands);
    let (tx,rx)=std::sync::mpsc::channel();
    capture.read_screen_rgba(buffer,vec![egui::ScreenshotCallback::new(move |image|{let _=tx.send(image);})]);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let image=rx.recv_timeout(Duration::from_secs(30)).unwrap();
    let pixels=image.pixels.iter().flat_map(|p|p.to_array()).collect();
    let image=photocraft_codecs::Image::from_u8(w,h,photocraft_codecs::ChannelLayout::Rgba,pixels).unwrap();
    let bytes=photocraft_codecs::encode(&image,photocraft_codecs::Format::Png,&Default::default()).unwrap();
    std::fs::write(root.join(format!("batch-{w}x{h}.png")),bytes).unwrap();
}

fn wait(app:&mut PhotocraftApp, ctx:&egui::Context) {
    let deadline=Instant::now()+Duration::from_secs(180);
    while app.ui.batch_converter.running {
        assert!(Instant::now()<deadline,"queue timeout");
        batch::tick(app,ctx);
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn main() {
    let root=PathBuf::from(std::env::args().nth(1).unwrap());
    let input=root.join("input"); let output=root.join("output");
    std::fs::create_dir_all(&input).unwrap();std::fs::create_dir_all(&output).unwrap();
    let a=input.join("źródło.jpg");let b=input.join("unchecked.jpg");let bad=input.join("broken.jpg");
    std::fs::copy(root.parent().unwrap().join("metadata/fixture.jpg"),&a).unwrap();
    std::fs::copy(&a,&b).unwrap();std::fs::write(&bad,b"broken JPEG").unwrap();
    let original=std::fs::read(&a).unwrap();
    let ctx=egui::Context::default();
    PhotocraftApp::setup_context(&ctx,Default::default());
    let mut app=PhotocraftApp::new(photocraft_engine::Session::new(),Services::default());
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"add","files":[bad,a,b]})).unwrap();
    assert_eq!(app.ui.batch_converter.entries.len(),3);
    app.ui.batch_converter.entries[2].enabled=false;
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"start","settings":{"folder":output,"format":"jxl","lossless":true,"metadata":"all"}})).unwrap();
    wait(&mut app,&ctx);
    assert_eq!(app.ui.batch_converter.entries[0].status,"Błąd");
    assert_eq!(app.ui.batch_converter.entries[1].status,"Gotowe","{}",app.ui.batch_converter.entries[1].message);
    assert_eq!(app.ui.batch_converter.entries[2].status,"Oczekuje");
    assert!(!output.join("unchecked.jxl").exists());
    assert_eq!(std::fs::read(&a).unwrap(),original);
    let jxl=app.ui.batch_converter.entries[1].output.clone();
    app.ui.batch_converter.entries.clear();
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"add","files":[jxl]})).unwrap();
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"start","settings":{"format":"jpg"}})).unwrap();
    wait(&mut app,&ctx);
    assert_eq!(app.ui.batch_converter.entries[0].status,"Gotowe","{}",app.ui.batch_converter.entries[0].message);
    assert_eq!(std::fs::read(&a).unwrap(),original);
    std::fs::write(root.join("result.json"),serde_json::to_vec_pretty(&json!({"passed":true,"jxl":jxl,"jpeg":app.ui.batch_converter.entries[0].output,"original_unchanged":true,"bad_input_continued":true,"unchecked_omitted":true})).unwrap()).unwrap();
    for (w,h) in [(1366.,768.),(1024.,640.)] {
        let mut bounds=None;
        let out=ctx.run_ui(egui::RawInput {screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(w,h))), ..Default::default()},|ui|{batch::show(&mut app,ui.ctx());});
        fn visit(s:&egui::epaint::Shape,b:&mut Option<egui::Rect>){match s {egui::epaint::Shape::Text(t) if t.galley.job.text.contains("Rozpocznij konwersję")=>*b=Some(t.visual_bounding_rect()),egui::epaint::Shape::Vec(v)=>for s in v {visit(s,b)},_=>{}}}
        for s in out.shapes {visit(&s.shape,&mut bounds);}
        println!("{w}x{h} Start bounds {bounds:?}");
        snapshot(&mut app,&root,w as u32,h as u32);
    }
    let mut result:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("result.json")).unwrap()).unwrap();
    app.ui.batch_converter.entries.clear();
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"add","files":[a,b]})).unwrap();
    app.ui.batch_converter.entries[1].enabled=false;
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"start","settings":{"folder":output,"format":"jxl","metadata":"none","lossless":true}})).unwrap();
    wait(&mut app,&ctx);
    let row=&app.ui.batch_converter.entries[0];assert_eq!(row.status,"Gotowe","{}",row.message);
    let image=photocraft_codecs::decode(&std::fs::read(&row.output).unwrap()).unwrap();
    assert!(image.meta.exif.is_none() && image.meta.xmp.is_none());
    result["metadata_none_strips_exif_xmp"]=json!(true);
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"start","settings":{"folder":input,"format":"jpg","metadata":"all","conflict":"overwrite"}})).unwrap();
    wait(&mut app,&ctx);
    assert_eq!(app.ui.batch_converter.entries[0].status,"Błąd");
    assert_eq!(std::fs::read(&a).unwrap(),original);
    assert_eq!(std::fs::read(&b).unwrap(),original);
    result["overwrite_cannot_replace_inputs"]=json!(true);
    app.ui.batch_converter.entries.clear();
    let sub=input.join("sub");std::fs::create_dir_all(&sub).unwrap();std::fs::copy(&a,sub.join("extra.JPG")).unwrap();
    std::fs::write(input.join("ignored.txt"),b"text").unwrap();
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"folder","path":input,"filter":"*.JPG","subfolders":false})).unwrap();wait(&mut app,&ctx);
    assert_eq!(app.ui.batch_converter.entries.len(),3);
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"folder","path":input,"filter":"*.JPG","subfolders":true})).unwrap();wait(&mut app,&ctx);
    assert_eq!(app.ui.batch_converter.entries.len(),4);
    result["folder_filters_recursion_deduplication"]=json!(true);
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"start","settings":{"folder":output,"format":"jxl","conflict":"unique"}})).unwrap();
    menus::invoke(&mut app,&ctx,"file.export.batchConverter",json!({"action":"cancel"})).unwrap();wait(&mut app,&ctx);
    assert!(!app.ui.batch_converter.running);
    assert!(app.ui.batch_converter.progress<1.0);
    assert_eq!(std::fs::read(&a).unwrap(),original);
    result["cancellation"]=json!(true);
    std::fs::write(root.join("result.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!("Production queue passed");
}
