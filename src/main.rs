use std::path::PathBuf;

use clap::Parser;
use cranelift_module::Module;
use cranelift_object::object::write::Symbol;

/// A compiler from `.clir` files to object files.
#[derive(Debug, clap::Parser)]
struct Args {
    /// The `.clir` file to compile.
    source: PathBuf,

    /// The path to the object file to output.
    #[clap(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    let source_text = std::fs::read_to_string(&args.source).unwrap();

    let functions = cranelift_reader::parse_functions(&source_text).unwrap();

    let shared_flags_builder = cranelift_codegen::settings::builder();
    let shared_flags = cranelift_codegen::settings::Flags::new(shared_flags_builder);

    let isa = cranelift_native::builder()
        .unwrap()
        .finish(shared_flags)
        .unwrap();

    let object_builder = cranelift_object::ObjectBuilder::new(
        isa,
        "main",
        cranelift_module::default_libcall_names(),
    )
    .unwrap();

    let mut object_module = cranelift_object::ObjectModule::new(object_builder);

    for function in functions {
        let func_id = object_module
            .declare_function(
                &function.name.to_string(),
                cranelift_module::Linkage::Export,
                &function.signature,
            )
            .unwrap();
        let mut ctx = cranelift_codegen::Context::for_function(function);
        object_module.define_function(func_id, &mut ctx).unwrap();
    }

    let object_product = object_module.finish();

    let output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(args.output)
        .unwrap();

    object_product.object.write_stream(output_file).unwrap();
}
