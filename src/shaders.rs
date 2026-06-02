pub const SOLID_VERTEX_SHADER: &str = r#"
    #version 140
    in vec3 position;
    in vec3 normal;
    in vec3 color;
    out vec3 v_color;
    out vec3 v_normal;
    out vec3 v_world_pos;
    uniform mat4 model;
    uniform mat4 vp;
    uniform mat4 normal_matrix;

    void main() {
        vec4 world = model * vec4(position, 1.0);
        v_color = color;
        v_normal = normalize((normal_matrix * vec4(normal, 0.0)).xyz);
        v_world_pos = world.xyz;
        gl_Position = vp * world;
    }
"#;

pub const SOLID_FRAGMENT_SHADER: &str = r#"
    #version 140
    in vec3 v_color;
    in vec3 v_normal;
    in vec3 v_world_pos;
    out vec4 f_color;
    uniform vec3 light_dir;
    uniform vec3 point_light_pos;
    uniform vec3 point_light_color;
    uniform float point_light_strength;
    uniform float time;
    uniform float alpha;

    void main() {
        float diffuse = max(dot(normalize(v_normal), normalize(light_dir)), 0.0);
        vec3 rim = vec3(0.08, 0.16, 0.26) * pow(1.0 - max(v_normal.z, 0.0), 2.0);
        vec3 to_light = point_light_pos - v_world_pos;
        float dist = max(length(to_light), 0.001);
        vec3 point_dir = to_light / dist;
        float point_diffuse = max(dot(normalize(v_normal), point_dir), 0.0);
        float attenuation = 1.0 / (1.0 + 0.24 * dist * dist);
        float flicker = 0.82 + 0.18 * sin(time * 4.6 + v_world_pos.x * 1.7 + v_world_pos.z * 1.9);
        vec3 point_light = point_light_color * point_diffuse * attenuation * point_light_strength * flicker;
        vec3 color = v_color * (0.28 + diffuse * 0.9) + rim + point_light;
        f_color = vec4(color, alpha);
    }
"#;

pub const STAR_VERTEX_SHADER: &str = r#"
    #version 140
    in vec3 position;
    in vec3 color;
    in float size;
    out vec3 v_color;
    uniform mat4 vp;
    uniform float time;
    uniform float layer_speed;
    uniform float wrap_depth;

    void main() {
        vec3 p = position;
        p.z = -18.0 - mod(abs(p.z) - 18.0 - time * layer_speed, wrap_depth);
        v_color = color;
        gl_PointSize = size * (1.0 + smoothstep(-18.0, -120.0, p.z));
        gl_Position = vp * vec4(p, 1.0);
    }
"#;

pub const STAR_FRAGMENT_SHADER: &str = r#"
    #version 140
    in vec3 v_color;
    out vec4 f_color;

    void main() {
        vec2 p = gl_PointCoord - vec2(0.5);
        float dist = dot(p, p);
        float alpha = smoothstep(0.25, 0.02, dist);
        f_color = vec4(v_color, alpha);
    }
"#;
