pub mod time;

trait Squarable {
    fn square(self) -> Self;
}
impl Squarable for f64 {
    fn square(self) -> Self {
        self * self
    }
}

pub fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, sphere_radius: f64) -> f64 {
    let lon1 = x0;
    let lon2 = x1;
    let lat1 = y0;
    let lat2 = y1;

    let d_lat = lat2 - lat1;
    let d_lon = lon2 - lon1;

    let a = (d_lat.to_radians() / 2.0).sin().square()
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * (d_lon.to_radians() / 2.0).sin().square();

    let c = 2.0 * a.sqrt().asin();

    let result = c * sphere_radius;
    return result;
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub type PointPair = (Point, Point);
