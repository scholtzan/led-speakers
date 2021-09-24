include <parameters.scad>;

$fn = 60;

module connector() {
    difference() {
        cylinder(connector_length, connector_outer / 2, connector_outer / 2);
        
        cylinder(connector_length, connector_inner / 2, connector_inner / 2);
    }
}

connector();