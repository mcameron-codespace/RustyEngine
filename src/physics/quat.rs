fn length_squared(q: [f32; 4]) -> f32 {
    q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]
}
pub fn normalize(q: [f32; 4]) -> [f32; 4] {
    let len_sq = length_squared(q);
    let len = len_sq.sqrt();
    if len == 0.0 {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        let inv = 1.0 / len;
        [q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv]
    }
}
pub fn slerp(q1: [f32; 4], q2: [f32; 4], alpha: f32) -> [f32; 4] {
    let mut q1 = normalize(q1);
    let mut q2 = normalize(q2);
    let mut dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];
    if dot < 0.0 {
        dot = -dot;
        q1 = [-q1[0], -q1[1], -q1[2], -q1[3]];
    }
    const DOT_THRESHOLD: f32 = 0.9995;
    let scale1: f32;
    let scale2: f32;
    if dot > DOT_THRESHOLD {
        scale1 = 1.0 - alpha;
        scale2 = alpha;
    } else {
        let theta_0 = dot.acos();
        let theta = theta_0 * alpha;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        scale1 = (theta_0 - theta).sin() / sin_theta_0;
        scale2 = sin_theta / sin_theta_0;
    }
    [
        scale1 * q1[0] + scale2 * q2[0],
        scale1 * q1[1] + scale2 * q2[1],
        scale1 * q1[2] + scale2 * q2[2],
        scale1 * q1[3] + scale2 * q2[3],
    ]
}
