include <parameters.scad>;

module led_segment() {
    // rail
    translate([0, 0, guide_distance]) {
        cube([guide_segment_length, guide_width, guide_thickness]);
    }
    
    // connection
    cube([guide_thickness, guide_width, guide_distance]);
    
    mirror([1, 0, 0]) {
        translate([0, guide_width, guide_distance]) {
            rotate([90, 0, 0]) {
                linear_extrude(height = guide_width) {
                    p0 = [-2, 0];
                    p1 = [0, -2];
                    p2 = [0, 0];
                    polygon([p0, p1, p2]);
                }
            }
        }
    }
        
    // foot
    intersection() {
        linear_extrude(height=guide_foot_length) {
            p0 = [guide_segment_length, -3];
            p1 = [0, 0];
            p2 = [0, 7];
            p3 = [guide_segment_length, 11];
            p4 = [guide_segment_length, 10];
            p5 = [0.5, 6];
            p6 = [0.5, 1];
            p7 = [guide_segment_length, -2];
            polygon([p0, p1, p2, p3, p4, p5, p6, p7]);
        }
        
        translate([0, guide_segment_length * 2, 0]) {
            rotate([90, 0, 0]) {
                linear_extrude(height=guide_segment_length * 3) {
                    p0 = [0, 0];
                    p1 = [0, guide_foot_length];
                    p2 = [guide_foot_length, guide_foot_length / 2];
                    p3 = [guide_segment_length - 3, 0];
                    polygon([p0, p1, p2, p3]);
                }
            }
        }
    }
}

module led_guide() {
    for (i = [0 : 1 : total_segments - 1]) {
        if (i % 2 == 0) {
            translate([-i * guide_segment_length, 0, 0]) {
                mirror([1, 0, 0]) {
                    led_segment();
                }
            }
        } else {
            translate([-(i + 1) * guide_segment_length, 0, 0]) {
                led_segment();
            }
        }
    }
}

//led_guide();

module guide_corner() {
    p0 = [0, 0];
    p1 = [led_strip_width, 0];
    p2 = [0, led_strip_width];
    
    linear_extrude(height=guide_width) {
        polygon([p0, p1, p2]);
    }

}

guide_corner();