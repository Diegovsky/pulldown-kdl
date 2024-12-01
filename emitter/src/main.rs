use std::path::Path;

use pulldown_kdl_emitter::KdlEmitter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name = std::env::args().nth(1).unwrap();
    let name = Path::new(&name);
    let doc = std::fs::read_to_string(name)?;
    let parser = pulldown_kdl::Parser::new(&doc);
    let mut buf = Vec::new();
    let mut emitter = KdlEmitter::new(parser);
    let result = emitter.emit(&mut buf);
    std::fs::write(name.with_extension("txt"), &buf)?;
    result?;
    Ok(())
}
