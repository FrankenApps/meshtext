use meshtext::{MeshText, TextSection};

/// Can be used to test the optimizer.
#[test]
#[allow(unused)]
fn optimizer_test() {
    let test_text = "Meshtext";
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    let mut generator = meshtext::MeshGenerator::new(font_data);
    let mesh: MeshText = generator
        .generate_section("Meshtext", true, None)
        .expect("Failed to generate text mesh for character.");

    let filename = format!("tests/optimizer_test_{}-original.svg", test_text);
    plot_vertices_svg(filename.as_str(), &mesh.vertices, false).unwrap();
}

/// Creates a SVG plot of the generated vertices.
///
/// Arguments:
///
/// * `path`: The output path to which the file will be written.
/// * `data`: The vertex positions of a glyph or text section.
/// * `optimized`: Wether the plotted vertices were optimized.
#[allow(unused)]
fn plot_vertices_svg(
    path: &str,
    data: &Vec<f32>,
    optimized: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    let root = SVGBackend::new(path, (4096, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let optimized_text = if optimized { "Optimized" } else { "Original" };
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Meshtext Triangulation ({})", optimized_text),
            ("sans-serif", 70),
        )
        .build_cartesian_2d(-0.025..3.975, -0.25..0.75)?;

    // Get triangles from vertices.
    let mut triangles: Vec<Vec<(f64, f64)>> = Vec::new();
    for t in data.chunks(9) {
        triangles.push(vec![
            (t[0] as f64, t[1] as f64),
            (t[3] as f64, t[4] as f64),
            (t[6] as f64, t[7] as f64),
        ]);
    }

    // Display the vertex and triangle count.
    let vertices_text = format!("Vertices: {}", data.len() / 3);
    let triangles_text = format!("Triangles: {}", triangles.len());
    root.draw_text(
        vertices_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 950),
    )?;
    root.draw_text(
        triangles_text.as_str(),
        &("sans-serif", 30).into_text_style(&root).color(&BLACK),
        (20, 980),
    )?;

    for triangle in triangles.iter_mut() {
        chart.draw_series(std::iter::once(Polygon::new(
            triangle.clone(),
            &RED.mix(0.2),
        )))?;
        triangle.push(triangle[0]);
        chart.draw_series(std::iter::once(PathElement::new(triangle.clone(), &RED)))?;
    }

    root.present().unwrap();
    Ok(())
}
