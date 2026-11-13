//! Every text entry point (pasted codes, saves from storage) must survive hostile input:
//! random garbage, truncations and single-character mutations of valid texts. No panic, no hang.
use turbocursa::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const ALPHA: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_;:,. \n\t%\xC3\xA9";

fn garbage(r: &mut Rng, len: usize) -> String {
    let b: Vec<u8> = (0..len).map(|_| ALPHA[r.below(ALPHA.len())]).collect();
    String::from_utf8_lossy(&b).into_owned()
}

fn feed(s: &str) {
    let _ = codes::parse(s);
    let _ = ghost::Ghost::decode(s);
    let _ = garage::Garage::decode(s);
    let _ = career::Career::from_save(s);
    let _ = champ::ChampCode::decode(s);
    let _ = share::StageCode::decode(s);
    let _ = settings::Settings::decode(s);
    let _ = profile::Profile::decode(s);
}

#[test]
fn hostile_text_never_panics() {
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    let prefixes = [
        "TCS1-", "TCC1-", "TG1;", "TG2;", "TG2", "TS1", "TC-", "TW-", "DAILY", "WEEKLY", "TCG",
    ];
    for i in 0..4000 {
        let len = r.below(if i % 50 == 0 { 3000 } else { 120 });
        let mut s = garbage(&mut r, len);
        if i % 3 == 0 {
            s = format!("{}{}", prefixes[r.below(prefixes.len())], s);
        }
        feed(&s);
    }
}

#[test]
fn truncations_and_mutations_of_valid_texts_survive() {
    let mut r = Rng(12345);
    let mut valid: Vec<String> = Vec::new();
    valid.push(garage::Garage::default().encode());
    valid.push(settings::Settings::default().encode());
    valid.push(career::Career::new(0, 7).to_save());
    for text in &valid {
        for n in 0..text.len().min(400) {
            if text.is_char_boundary(n) {
                feed(&text[..n]);
            }
        }
        for _ in 0..600 {
            let mut b = text.clone().into_bytes();
            let k = 1 + r.below(4);
            for _ in 0..k {
                if b.is_empty() {
                    break;
                }
                let i = r.below(b.len());
                b[i] = ALPHA[r.below(ALPHA.len())];
            }
            feed(&String::from_utf8_lossy(&b));
        }
    }
}
