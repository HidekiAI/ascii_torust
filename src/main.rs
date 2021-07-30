// Original author: Andy Sloane (https://github.com/a1k0n/)
// Writeup: https://www.a1k0n.net/2011/07/20/donut-math.html
// Actual C code:
//              k;double sin()
//          ,cos();main(){float A=
//        0,B=0,i,j,z[1760];char b[
//      1760];printf("\x1b[2J");for(;;
//   ){memset(b,32,1760);memset(z,0,7040)
//   ;for(j=0;6.28>j;j+=0.07)for(i=0;6.28
//  >i;i+=0.02){float c=sin(i),d=cos(j),e=
//  sin(A),f=sin(j),g=cos(A),h=d+2,D=1/(c*
//  h*e+f*g+5),l=cos      (i),m=cos(B),n=s\
// in(B),t=c*h*g-f*        e;int x=40+30*D*
// (l*h*m-t*n),y=            12+15*D*(l*h*n
// +t*m),o=x+80*y,          N=8*((f*e-c*d*g
//  )*m-c*d*e-f*g-l        *d*n);if(22>y&&
//  y>0&&x>0&&80>x&&D>z[o]){z[o]=D;;;b[o]=
//  ".,-~:;=!*#$@"[N>0?N:0];}}/*#****!!-*/
//   printf("\x1b[H");for(k=0;1761>k;k++)
//    putchar(k%80?b[k]:10);A+=0.04;B+=
//      0.02;}}/*****####*******!!=;:~
//        ~::==!!!**********!!!==::-
//          .,~~;;;========;;;:~-.
//              ..,--------,*/
//
// Port: Hideki A. Ikeda <HidekiAI@CodeMonkeyNinja.dev>
// 2021-July

//use std::ptr;
use std::{thread, time}; // sleep

// Assumption:
// * Torus inner radius = 1
// * Torus outer radius = 2
// * Torus Z position = 10
// * Y-up, X-right, Z-forward
fn main() {
    let pixels = b".,-~:;=!*#$@"; // gets darker as surface normal vector gets longer

    // Calculate K1 based on screen size: the maximum x-distance occurs
    // roughly at the edge of the torus, which is at x=R1+R2, z=0.  we
    // want that to be displaced 3/8ths of the width of the screen, which
    // is 3/4th of the way from the center to the side of the screen.
    // screen_width*3/8 = K1*(R1+R2)/(K2+0)
    // screen_width*K2*3/(8*(R1+R2)) = K1
    let screen_width = 80.0f32;
    let screen_height = 22.0;
    //let screen_center_x = screen_width / 2.0;
    //let screen_center_y = screen_height / 2.0;
    let char_space = b" "[0]; //32u8;
    let char_line_feed = 10u8; // 10 (0x0A) is ctrl-J (Linefeed/newline), see also decimal13 (0x0D) carriage-return
    let screen_dim = (screen_width * (screen_height + 1.0)) as usize;
    //let size_of_f32 = std::mem::size_of::<f32> as usize;
    //let torus_midpoint_z = 10.0;
    let theta_rotate_steps = 0.07;
    let phi_rotate_steps = 0.02;
    //let inner_radius = 1.0; // aka R1
    //let outer_radius = 2.0; // aka R2
    let two_pi = 2.0 * ::std::f32::consts::PI; //3.14159265;

    // FoV: (x', y') = (K1x/K2+z, K1y/K2+z)
    //let midpoint_screen_projection_z = torus_midpoint_z / 2.0; // aka K2

    //let kprojection_screen_midpoint_xz = screen_width * midpoint_screen_projection_z * 3.0 / (8.0 * (inner_radius + outer_radius)); // aka K1: distance z' is constant, arbitrarily based on FoV

    let mut mut_a = 0.0f32;
    let mut mut_b = 0.0f32;
    let mut mut_phi = 0.0f32;
    let mut mut_theta = 0.0f32;
    let dim_size = screen_dim + (screen_width as usize) + 2; // add one extra line
    let mut mut_z_buff = vec![0.0f32; dim_size];
    let mut mut_render_buff = vec![char_space; dim_size];
    println!("\x1b[2J"); // ESC[2J = clear entire screen (CLS)
    loop {
        // clear buffers
        // NOTE: Will switch to unsafe memset once all is working
        // unsafe {
        //     // mimic memset
        //     let b_ptr = mut_render_buff.as_mut_ptr();
        //     let z_ptr = mut_z_buff.as_mut_ptr();
        //     ptr::write_bytes(b_ptr, char_space, screen_dim); // 32=ASCII<space>
        //     ptr::write_bytes(z_ptr, 0, screen_dim * size_of_f32);
        //     // 7040 = 1760 * 4
        // }
        for buff_index in 0..screen_dim {
            mut_render_buff[buff_index] = char_space;
            mut_z_buff[buff_index] = 0.0;
        }
        // precompute cosines and sines of A, B, theta, phi, same as before
        let sin_a = mut_a.sin(); // aka e
        let cos_a = mut_a.cos(); // aka g
        let cos_b = mut_b.cos(); // aka m
        let sin_b = mut_b.sin(); // aka n

        // thata goes around the cross-sectional circle of a torus
        mut_theta = 0.0;
        //for theta in (0.0..two_pi).step_by(theta_rotate_steps) causes range issue due to floating point accuracies, hence we'll just loop and do inc ourself
        loop {
            // Rotate about 1 axis:
            // (x, y, z) = (R2, 0, 0) + (R1 cos(theta), R1 sin(theta), 0)
            // (R2 + R1 cos(theta), R1 sin(theta), 0) * | cos(phi)  0  sin(phi) |
            //                                          |      0    1      0    |
            //                                          | -sin(phi) 0  cos(phi) |
            //  = ((R2 + R1 cos(theta) cos(phi)), R1 sin(theta), -(R2 + R1 cos(theta)) sin(phi))
            //  Rotate about 2 axis:
            //  (R2 + R1 cos(theta), R1 sin(theta), 0) * | cos(phi)  0  sin(phi) |   | 1   0       0    |   | cos(B)  sin(B) 0 |
            //                                           |    0      1     0     | * | 0 cos(A)  sin(A) | * | -sin(B) cos(B) 0 |
            //                                           | -sin(phi) 0  cos(phi) |   | 0 -sin(A) cos(A) |   | 0         0    1 |

            // var d=Math.cos(j), f=Math.sin(j); // cosine theta, sine theta
            let cos_theta = mut_theta.cos();
            let sin_theta = mut_theta.sin();
            // phi goes around the center of revolution of a torus
            mut_phi = 0.0;
            loop {
                // var xp=(150 + K1 * ooz * x); // x' = screen space coordinate, translated and scaled to fit our 320x240 canvas element
                // var yp=(120 - K1 * ooz * y); // y' (it's negative here because in our output, positive y goes down but in our 3D space, positive y goes up)
                // // luminance, scaled back to 0 to 1
                // var L=0.7 * (cos_phi * cos_theta * sB - cA * ct * sp - sA * sin_theta + cB * (cA * st - ct * sA * sp));
                // if(L > 0) {
                //   ctx.fillStyle = 'rgba(255,255,255,'+L+')';
                //   ctx.fillRect(xp, yp, 1.5, 1.5);
                // }

                // var c=Math.sin(i),l=Math.cos(i); // cosine phi, sine phi
                let sin_phi = mut_phi.sin();
                let cos_phi = mut_phi.cos();

                // the x,y coordinate of the circle, before revolving
                //let ox = outer_radius + inner_radius * cos_theta; // R2 + R1 * ct => (R2,0,0) + (R1 cos(theta), R1 sin(theta), 0)
                //let oy = inner_radius * sin_theta; // R1 * st

                // var x = ox * (cB * cos_phi + sA * sB * sin_phi) - oy * cA * sB; // final 3D x coordinate
                // var y = ox * (sB * cos_phi - sA * cB * sin_phi) + oy * cA * cB; // final 3D y
                //let proj_z = midpoint_screen_projection_z + cos_a * ox * sin_phi + sin_a * oy; //  K2 + cos_a * circlex * sinphi + circley * sin_a

                // x = (R2 + R1 cos(theta))(cos(B) cos(phi) + sin(A) sib(B) sin(phi)) - R1 cos(A) sin(B) sin(theta)
                // y = (R2 + R1 cos(theta))(cos(phi) sin(B) - cos(B) sin(A) sin(phi)) + R1 cos(A) cos(B) sin(theta)
                // z = cos(A)(R2 + R1 cos(theta)) sin(phi) + R1 sin(A) sin(theta)
                // float h = cos_theta + 2;
                //let h2 = cos_theta * (outer_radius + inner_radius); // R1 + R2 * cos(theta) ; also see h=d+2
                let h = cos_theta + 2.0; // R1 + R2 * cos(theta) ; also see h=d+2

                // float t = sin_phi * h * cos_a - sin_theta * sin_a;
                let _t = sin_phi * h * cos_a - sin_theta * sin_a;
                // float D = 1 / (sin_phi * h * sin_a + sin_theta * cos_a + 5);
                let _D = 1.0 / (sin_phi * h * sin_a + sin_theta * cos_a + 5.0);
                //let local_z = sin_phi * h * sin_a + sin_theta * cos_a + midpoint_screen_projection_z; // +5 to adjust midpoint (also makes it so it won't cause 1/z (1/0) infinity)
                //let one_over_z = 1.0 / local_z; // 1/z == 0 is infinite depth
                //let t = sin_phi * h * cos_a - sin_theta * sin_a; // "this is a clever factoring of some of the terms in x' and y'"

                // int x = 40 + 30 * D * (cos_phi * h * cos_b - t * sin_b);
                // x = 40 + 30 * D * (l * h * m - t * n)
                let _x = (40.0 + 30.0 * _D * (cos_phi * h * cos_b - _t * sin_b)) as usize;
                //let projection_x = // circlex * (cos_b * cosphi + sin_a * sin_b * sinphi) - circley * cos_a * sin_b
                //    screen_center_x + 30.0 * one_over_z * (cos_phi * h * cos_b - t * sin_b);
                // int y = 12 + 15 * D * (cos_phi * h * sin_b + t * cos_b);
                // y = 12 + 15 * D * (l * h * n + t * m)
                let _y = (12.0 + 15.0 * _D * (cos_phi * h * sin_b + _t * cos_b)) as usize;
                //let projection_y =
                //    // circlex * (sin_b * cosphi - sin_a * cos_b * sinphi) + circley * cos_a * cos_b
                //    screen_center_y + 15.0 * one_over_z * (cos_phi * h * sin_b + t * cos_b);
                //let ooz_proj = 1.0 / proj_z; // one over z (ooz)

                // surface normal
                //let pos_xy = projection_x + (screen_width * projection_y); // go down y rows, and offset by x
                //let us_pos_xy = pos_xy as usize; //if posXY >= 1760.0 { 0.0 } else { posXY };
                // float L = cosphi * costheta * sin_b - cos_a * costheta * sinphi - sin_a * sintheta + cos_b * (cos_a * sintheta - costheta * sin_a * sinphi);
                // L ranges from -sqrt(2) to +sqrt(2).  If it's < 0, the surface is pointing away from us, so we won't bother trying to plot it.
                //let luminance = (sin_theta * sin_a - sin_phi * cos_theta * cos_a) * cos_b
                //    - sin_phi * cos_theta * sin_a
                //    - sin_theta * cos_a
                //    - cos_phi * cos_theta * sin_b;
                // luminance_index is now in the range 0..11 (8*sqrt(2) = 11.3), now we lookup the character corresponding to the luminance and plot it in our output:
                //let luminance_index = 8.0 * luminance; // range it between 0..11
                //let depth = mut_z_buff[us_pos_xy];
                //// calculate luminance. ugly, but correct.
                //if screen_height > projection_y
                //    && projection_y > 0.0
                //    && projection_x > 0.0
                //    && screen_width > projection_x
                //    && one_over_z > depth
                //// test against the z-buffer.  larger 1/z means the pixel is closer to the viewer than what's already plotted.
                //{
                //    //mut_z_buff[us_pos_xy] = one_over_z;
                //    let index = match luminance_index {
                //        _ if luminance_index > 0.0 => luminance_index as usize,
                //        _ => 0 as usize, // no light
                //    };
                //    //mut_render_buff[us_pos_xy] = pixels[index];
                //}
                ////else {
                ////    // until we get clear buffer working, we'll just fill in the array...
                ////    mut_render_buff[us_pos_xy] = char_space;
                ////}

                // int o = x + 80 * y;
                // int N = 8 * ((sin_theta * sin_a - sin_phi * cos_theta * cos_a) * cos_b - sin_phi * cos_theta * sin_a - sin_theta * cos_a - cos_phi * cos_theta * sin_b);     // luminance_index is now in the range 0..11 (8*sqrt(2) = 11.3), now we lookup the character corresponding to the luminance and plot it in our output:
                // if(22 > y && y > 0 && x > 0 && 80 > x && D > z[o]) {
                //   z[o] = D;
                //   b[o] = ".,-~:;=!*#$@"[N > 0 ? N : 0];
                // }
                let _o = _x + 80 * _y;
                if _o >= mut_render_buff.len() {
                    panic!("Index for {},{} exceeds {}", _x, _y, mut_render_buff.len());
                }
                let _N = 8.0
                    * ((sin_theta * sin_a - sin_phi * cos_theta * cos_a) * cos_b
                        - sin_phi * cos_theta * sin_a
                        - sin_theta * cos_a
                        - cos_phi * cos_theta * sin_b); // luminance_index is now in the range 0..11 (8*sqrt(2) = 11.3), now we lookup the character corresponding to the luminance and plot it in our output:
                if 22 > _y && _y > 0 && _x > 0 && 80 > _x && _D > mut_z_buff[_o] {
                    mut_z_buff[_o] = _D;
                    let _bi = if _N > 0.0 { _N as usize } else { 0 as usize };
                    let _pixel = pixels[_bi];
                    mut_render_buff[_o] = _pixel;
                }
                // else {
                //     // until we get clear buffer working, we'll just fill in the array...
                //     mut_render_buff[_o] = char_space;
                // }

                // next phi
                mut_phi += phi_rotate_steps;
                if mut_phi > two_pi {
                    break;
                };
            }
            mut_theta += theta_rotate_steps;
            if mut_theta > two_pi {
                break;
            };
        }
        // flush before moving cursor to HOME, or else last line appaears at the top...
        unsafe {
            ::libc::putchar(char_line_feed.into());
        }
        println!("\x1b[H"); // ESC[H = move cursor back to HOME position
        for screen_index in 0..(screen_dim + 1) {
            // +1 so that final corner gets line-feed
            let ch = match screen_index {
                _ if (screen_index % (screen_width as usize)) != 0 => mut_render_buff[screen_index],
                _ => char_line_feed,
            };
            unsafe {
                ::libc::putchar(ch.into());
            }
            mut_a += 0.00004;
            mut_b += 0.00002;
        }
        let ms = time::Duration::from_millis(300);
        thread::sleep(ms);
    }
}
