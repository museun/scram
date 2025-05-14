use super::Channel;

#[profiling::function]
pub fn apply_rfft(channel: &mut Channel) {
    let _ = microfft::real::rfft_2048(&mut channel.fft_input);
}
