use crate::types::{SsrcIgnoredMap, SsrcLastPktIdMap, SsrcStreamMap};

use songbird::packet::rtp::RtpType;

const SIZE_OF_I16: usize = std::mem::size_of::<i16>();

pub async fn voice_packet(
    audio: Option<Vec<i16>>,
    ssrc: u32,
    payload_type: RtpType,
    sequence: u16,
    ssrc_stream_map: SsrcStreamMap,
    ssrc_ignored_map: SsrcIgnoredMap,
    ssrc_last_pkt_id_map: SsrcLastPktIdMap,
) {
    if ssrc_ignored_map.get(&ssrc).map_or(false, |x| *x.value()) {
        return;
    }

    // TODO: if current - 1 is in the out of order pkts, add it before this one
    if let Some(mut pkt_id) = ssrc_last_pkt_id_map.get_mut(&ssrc) {
        let expected = *pkt_id.value() + 1;
        if expected != sequence {
            warn!(
                ?ssrc
                "got out of order audio packet! expected {}, got {}",
                expected, sequence
            );
            *pkt_id.value_mut() = sequence + 1;
            return;
        } else {
            *pkt_id.value_mut() = expected;
        }
    } else {
        ssrc_last_pkt_id_map.insert(ssrc, sequence);
    }

    if let (Some(audio), Some(stream)) = (audio, ssrc_stream_map.get(&ssrc)) {
        debug!(%ssrc, "got {} bytes of audio", audio.len() * SIZE_OF_I16);
        let (sample_rate, stereo) = packet_type_to_data(payload_type);

        debug!("processing audio");
        let audio = scripty_audio::process_audio(&audio, sample_rate, stereo, 16_000.0);

        debug!("feeding audio to stream");
        stream.feed_audio_async(audio).await;
        debug!("done processing pkt");
    }
}

/// Given a packet type, return a 2-tuple:
/// 0. sample rate in Hz
/// 1. whether this is stereo audio
fn packet_type_to_data(pkt_type: songbird::packet::rtp::RtpType) -> (f64, bool) {
    match pkt_type {
        RtpType::Pcmu
        | RtpType::Gsm
        | RtpType::G723
        | RtpType::Dvi4(5)
        | RtpType::Lpc
        | RtpType::Pcma
        | RtpType::G722
        | RtpType::Qcelp
        | RtpType::Cn
        | RtpType::G728
        | RtpType::G729 => (8000.0, false),
        RtpType::Dvi4(6) => (16000.0, false),
        RtpType::Dvi4(16) => (11025.0, false),
        RtpType::Dvi4(17) => (22050.0, false),
        RtpType::L16Stereo => (44100.0, false),
        RtpType::L16Mono => (44100.0, true),
        RtpType::Mpa => (90000.0, false),
        _ => {
            debug!(
                "got invalid pkt type {:?}, defaulting to 48KHz stereo",
                pkt_type
            );
            (48000.0, true)
        }
    }
}
