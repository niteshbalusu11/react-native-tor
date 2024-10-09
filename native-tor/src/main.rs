use tor::{self, run_arti};

fn main() -> anyhow::Result<()> {
    let res = run_arti("example.com", "/Users/nitesh/Documents/react-native-lnd-tor/native-tor")?;

    println!("result of running tor is {:?}", res);

    Ok(())
}
