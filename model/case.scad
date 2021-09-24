// Case for Raspberry Pi

include <parameters.scad>;

$fn = 30;


module case_outer_shell() {
    difference() {
        hull() {
            translate([case_bevel_diameter / 2, case_bevel_diameter / 2]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2, case_bevel_diameter / 2);
            }
            
            translate([case_width, case_bevel_diameter / 2]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2, case_bevel_diameter / 2);
            }
            
            translate([case_bevel_diameter / 2, case_length]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2, case_bevel_diameter / 2);
            }
            
            translate([case_width, case_length]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2, case_bevel_diameter / 2);
            }
        }
        
        hull() {
            translate([case_bevel_diameter / 2 + case_thickness, case_bevel_diameter / 2 + case_thickness, case_thickness]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2 - case_thickness / 2, case_bevel_diameter / 2 - case_thickness / 2);
            }
            
            translate([case_width - case_thickness, case_bevel_diameter / 2 + case_thickness, case_thickness]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2 - case_thickness / 2, case_bevel_diameter / 2 - case_thickness / 2);
            }
            
            translate([case_bevel_diameter / 2 + case_thickness, case_length - case_thickness, case_thickness]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2 - case_thickness / 2, case_bevel_diameter / 2 - case_thickness / 2);
            }
            
            translate([case_width - case_thickness, case_length - case_thickness, case_thickness]) {
                cylinder(case_height / 2 + case_thickness, case_bevel_diameter / 2 - case_thickness / 2, case_bevel_diameter / 2 - case_thickness / 2);
            }
        }
    }
}

module case_connectors() {
    translate([case_bevel_diameter, case_thickness, 0]) {
            cube([case_width - 2 * case_bevel_diameter, case_thickness / 2, case_height / 2 + case_thickness + case_connector_length]);
        }
        
        translate([case_bevel_diameter, case_thickness / 2 + case_length, 0]) {
            cube([case_width - 2 * case_bevel_diameter, case_thickness / 2, case_height / 2 + case_thickness + case_connector_length]);
        }
        
        translate([case_thickness, case_bevel_diameter, 0]) {
            cube([case_thickness / 2, case_length - 2 * case_bevel_diameter, case_height / 2 + case_thickness + case_connector_length]);
        }
        
        translate([case_thickness / 2 + case_width, case_bevel_diameter, 0]) {
            cube([case_thickness / 2, case_length - 2 * case_bevel_diameter, case_height / 2 + case_thickness + case_connector_length]);    
        }
}

module case_half(is_top) {
    if (is_top) {
        case_outer_shell();
        case_connectors();
    } else {
        difference() {
            translate([0, 0, case_height + case_thickness]) {
                mirror([0, 0, 1]) {
                    case_outer_shell();
                }
            }
            case_connectors();
        }
    }
}

module case_shell(is_top) {
    difference() {
        case_half(is_top);
        
        // power connector hole
    
        translate([-case_thickness, case_length - power_connector_dist + power_connector_diameter / 2, case_height / 2 + case_thickness]) {
            rotate([0, 90, 0]) {
                cylinder(case_thickness * 3, power_connector_diameter / 2, power_connector_diameter / 2);
            }
        }
        
        // LED connector
        translate([-case_thickness, case_length - led_connector_dist, case_height / 2 + case_thickness - led_connector_height / 2]) {
            cube([case_thickness * 3, led_connector_width, led_connector_height]);
        }
        
        // USB
        translate([case_width - usb_width - usb_dist + case_thickness, case_length, case_height / 2 + case_thickness - usb_height / 2]) {
            cube([usb_width, case_thickness * 3, usb_height]);
        }
    }
}

module case_top() {
    case_shell(true);
}

module case_bottom() {
    case_shell(false);
}


case_top();
//case_bottom();