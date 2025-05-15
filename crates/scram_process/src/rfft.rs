#[profiling::function]
pub fn apply_rfft(data: &mut [f32]) {
    macro_rules! make_rfft_size {
        ($($size:expr => $func:ident)*) => {
            match data.len() {
                $($size => {
                    let Ok(data) = <_>::try_from(data) else {
                        unreachable!()
                    };
                    _ = microfft::real::$func(data);
                })*,
                _ => unreachable!()
            }
        };
    }

    make_rfft_size! {
        32 => rfft_32
        64 => rfft_64
        128 => rfft_128
        256 => rfft_256
        512 => rfft_512
        1024 => rfft_1024
        2048 => rfft_2048
        4096 => rfft_4096
    }
}
