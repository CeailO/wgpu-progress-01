# Primitive Points and Line

Notable Primitive Context

    topology: PrimitiveTopology::LineStrip, // LineList // Pointlist
    strip_index_format: Some(IndexFormat::Uint32), // None // None

Render

    render_pass.draw(0..6, 0..6) // 3
