pub fn clamp<T>(x: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if x > min {
        if x > max {
            max
        } else {
            x
        }
    } else {
        min
    }
}
