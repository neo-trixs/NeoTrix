//! spike: ort 加载 sherpa whisper-tiny encoder 并跑一次前向（形状验证）。
//! 需 libonnxruntime.1.28.0.dylib 与二进制同目录（或 ORT_DYLIB_PATH）；
//! 已在隔离 crate 验证通过（见 spike 结论），仓库内跑需先修 onnx feature
//! 下他窗 ort-1.x 旧调用（ocr/image_super_resolution）。
//!
//! 运行（项目根）：
//! ```sh
//! cargo run -p neotrix --features onnx --example nt_whisper_spike
//! ```
//! 模型：datasets/_models/sherpa-onnx-whisper-tiny.en/tiny.en-encoder.int8.onnx
//! 实测：session 0.3s，前向 0.1s，输出 n_layer_cross_k/v [4,1,1500,384]。

use ort::session::Session;
use ort::value::Tensor;

fn main() -> ort::Result<()> {
    let t0 = std::time::Instant::now();
    let mut session = Session::builder()?.commit_from_file(
        "datasets/_models/sherpa-onnx-whisper-tiny.en/tiny.en-encoder.int8.onnx",
    )?;
    println!("session loaded in {:.1}s", t0.elapsed().as_secs_f32());
    // 2026-09-30: 原来读 `input.name` / `input.input_type` / `output.output_type`。
    // ort 新版把 `Outlet` 的这些字段收成私有 ⇒ E0616 私有字段 + E0609 字段不存在。
    // 改走 Debug：形状验证要的是「有哪些 input/output」，Debug 已经给全，
    // 且不把编译绑死在 ort 的字段可见性上。
    for input in session.inputs().iter() {
        println!("input: {input:?}");
    }
    for output in session.outputs().iter() {
        println!("output: {output:?}");
    }
    // 30s 静音等价输入：全 -1.5（与 nt_speech_transcribe 静音基线一致）
    let mel = vec![-1.5f32; 1 * 80 * 3000];
    let tensor = Tensor::from_array((vec![1usize, 80, 3000], mel))?;
    let t1 = std::time::Instant::now();
    let outputs = session.run(ort::inputs![tensor])?;
    println!("forward done in {:.1}s", t1.elapsed().as_secs_f32());
    for (name, value) in outputs.iter() {
        let (shape, _data) = value.try_extract_tensor::<f32>()?;
        println!("{name}: shape={shape:?}");
    }
    Ok(())
}
