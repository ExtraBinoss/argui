fn refracted_backdrop(pixel: vec2<f32>) -> vec4<f32> {
    let center = params.bounds.xy + params.bounds.zw * 0.5;
    let direction = pixel - center;
    let warped = pixel - direction * params.data.x * 0.08;
    let shift = direction * params.data.y * 0.01;
    return vec4<f32>(
        sample_backdrop(warped + shift).r,
        sample_backdrop(warped).g,
        sample_backdrop(warped - shift).b,
        1.0
    );
}
