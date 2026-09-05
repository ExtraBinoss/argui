use argui_paint::{EffectInstance, Filter, Refraction};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PlannedFilter {
    Blur(f32),
    ColorMatrix([f32; 20]),
    Refraction(Refraction),
    Effect(EffectInstance),
}

pub(crate) fn plan_filters(filters: &[Filter]) -> Vec<PlannedFilter> {
    let mut planned = Vec::new();
    let mut color = None;
    for filter in filters {
        if matches!(filter, Filter::Blur(radius) if *radius <= 0.01) {
            continue;
        }
        if let Some(matrix) = color_matrix(filter) {
            color = Some(compose(matrix, color.unwrap_or_else(identity)));
            continue;
        }
        flush_color(&mut planned, &mut color);
        planned.push(match filter {
            Filter::Blur(radius) => PlannedFilter::Blur(*radius),
            Filter::Refraction(value) => PlannedFilter::Refraction(*value),
            Filter::Effect(effect) => PlannedFilter::Effect(effect.clone()),
            Filter::Brightness(_)
            | Filter::Contrast(_)
            | Filter::Saturation(_)
            | Filter::HueRotate(_)
            | Filter::Opacity(_)
            | Filter::ColorMatrix(_) => unreachable!("color filters were handled above"),
        });
    }
    flush_color(&mut planned, &mut color);
    planned
}

fn flush_color(planned: &mut Vec<PlannedFilter>, color: &mut Option<[f32; 20]>) {
    if let Some(matrix) = color.take() {
        planned.push(PlannedFilter::ColorMatrix(matrix));
    }
}

fn color_matrix(filter: &Filter) -> Option<[f32; 20]> {
    match filter {
        Filter::Brightness(value) => Some(diagonal([*value, *value, *value, 1.0])),
        Filter::Contrast(value) => {
            let mut matrix = diagonal([*value, *value, *value, 1.0]);
            let offset = 0.5 * (1.0 - value);
            matrix[16..19].fill(offset);
            Some(matrix)
        }
        Filter::Saturation(value) => Some(saturation(*value)),
        Filter::HueRotate(angle) => Some(hue_rotation(*angle)),
        Filter::Opacity(value) => Some(diagonal([1.0, 1.0, 1.0, *value])),
        Filter::ColorMatrix(matrix) => Some(*matrix),
        Filter::Blur(_) | Filter::Refraction(_) | Filter::Effect(_) => None,
    }
}

fn identity() -> [f32; 20] {
    diagonal([1.0; 4])
}

fn diagonal(values: [f32; 4]) -> [f32; 20] {
    let mut matrix = [0.0; 20];
    for (index, value) in values.into_iter().enumerate() {
        matrix[index * 4 + index] = value;
    }
    matrix
}

fn saturation(value: f32) -> [f32; 20] {
    let inverse = 1.0 - value;
    let [red, green, blue] = [0.2126 * inverse, 0.7152 * inverse, 0.0722 * inverse];
    [
        red + value,
        green,
        blue,
        0.0,
        red,
        green + value,
        blue,
        0.0,
        red,
        green,
        blue + value,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ]
}

fn hue_rotation(angle: f32) -> [f32; 20] {
    let (sine, cosine) = angle.sin_cos();
    [
        0.213 + cosine * 0.787 - sine * 0.213,
        0.715 - cosine * 0.715 - sine * 0.715,
        0.072 - cosine * 0.072 + sine * 0.928,
        0.0,
        0.213 - cosine * 0.213 + sine * 0.143,
        0.715 + cosine * 0.285 + sine * 0.140,
        0.072 - cosine * 0.072 - sine * 0.283,
        0.0,
        0.213 - cosine * 0.213 - sine * 0.787,
        0.715 - cosine * 0.715 + sine * 0.715,
        0.072 + cosine * 0.928 + sine * 0.072,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
    ]
}

fn compose(after: [f32; 20], before: [f32; 20]) -> [f32; 20] {
    let mut output = [0.0; 20];
    for row in 0..4 {
        for column in 0..4 {
            output[row * 4 + column] = (0..4)
                .map(|middle| after[row * 4 + middle] * before[middle * 4 + column])
                .sum();
        }
        output[16 + row] = after[16 + row]
            + (0..4)
                .map(|middle| after[row * 4 + middle] * before[16 + middle])
                .sum::<f32>();
    }
    output
}
