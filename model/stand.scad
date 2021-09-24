// Speaker stand

include <parameters.scad>;

$fn = 15;

module base() {
    hull() {
        cylinder(r=2, h=standWidth);
        
        translate([standLength * 0.25, standHeight, 0]) {
            cylinder(r=5, h=standWidth);
        }
        
        translate([standLength, standHeight * 1/6, 0]) {
            cube([1, 1, standWidth]);
        }
        
        translate([standLength, 0, 0]) {
            cube([1, 1, standWidth]);
        }
    }
    
    translate([standLength + 1, 0, 0]) {
        hull() {
            translate([0, 2, 0]) {
                cylinder(r=2, h=standWidth);
            }
            
            translate([1, standRimHeight, 0]) {
                cylinder(r=1, h=standWidth);
            }
        }
    }
}

module sideBevelTool() {
    translate([0, standHeight * 1.5, 0]) {
        rotate([90, 0, 0]){
            linear_extrude(height=standHeight*2) {
                p0 = [0, 0];
                p1 = [standLength + standRimWidth, 0];
                p2 = [standLength + standRimWidth, standWidth * 0.25];
                
                polygon([p0, p1, p2]);
            }
        }
    }
}

module speakerStand() {
    rotate([0, 0, 6]) {
        difference() {
            base();
            sideBevelTool();
            
            translate([0, 0, standWidth]) {
                mirror([0, 0, -1]) {
                    sideBevelTool();
                }
            }
            
            // center hole
            translate([0, standHeight * 1.5, 0]) {
                rotate([90, 0, 0]) {
                    hull() {
                        translate([-10, standWidth * 0.3, 0]) {
                            cylinder(r=5, h=standHeight*2);
                        }
                        
                        translate([-10, standWidth * 0.7, 0]) {
                            cylinder(r=5, h=standHeight*2);
                        }
                        
                        translate([standLength * 0.5, standWidth * 0.4, 0]) {
                            cylinder(r=5, h=standHeight*2);
                        }
                        
                        translate([standLength * 0.5, standWidth * 0.6, 0]) {
                            cylinder(r=5, h=standHeight*2);
                        }
                    }  
                }
            }
        }
    }
    
}

speakerStand();