//#[path = "../src/fms.rs"]
//mod fms;

use async_std::task::sleep;
use clap::{App, Arg};
use futures::prelude::*;
use futures::select;
use std::convert::TryFrom;
use std::time::Duration;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
use protobuf::Message;
use fms::VehicleStatus;
use std::ops::Deref;
//use zenoh_protobuf_vehicle::fms::VehicleStatus;
use zenoh_protobuf_vehicle::fms;

#[async_std::main]
async fn main() {
    // Initiate logging
    env_logger::init();

    let (config, key_expr) = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    println!("Declaring Subscriber on '{}'...", &key_expr);

    let subscriber = session.declare_subscriber(&key_expr).res().await.unwrap();

    println!("Enter 'q' to quit...");
    let mut stdin = async_std::io::stdin();
    let mut input = [0_u8];
    loop {
        select!(
            sample = subscriber.recv_async() => {
                let sample = sample.unwrap();
                /* println!(">> [Subscriber] Received {} ('{}': '{}')",
                    sample.kind, sample.key_expr.as_str(), sample.value); */
                    
            	let _protobuff = <VehicleStatus as Message>::parse_from_bytes(&*sample.value.payload.contiguous());
            	//let _protobuff: zenoh_protobuf_vehicle::fms::VehicleStatus = Message::parse_from_bytes(&*sample.value.payload.contiguous());
            	//out_bytes: Vec<u8> = sample.value.payload.contiguous();
            	//let _protobuff: VehicleStatus = Message::parse_from_bytes(&*sample.value.payload.contiguous());
            	//let _protobuff: fms::VehicleStatus = Message::parse_from_bytes(sample.value.payload.contiguous().deref());
            	println!("{:#?}", _protobuff);
            	println!("#########################");
            	
            },
            
            _ = stdin.read_exact(&mut input).fuse() => {
            
                match input[0] {
                    b'q' => break,
                    0 => sleep(Duration::from_secs(1)).await,
                    _ => (),
                }
                
            }
        );
    }
}

fn parse_args() -> (Config, KeyExpr<'static>) {
    let args = App::new("zenoh sub example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...   'Endpoints to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(
            Arg::from_usage("-k, --key=[KEYEXPR] 'The key expression to subscribe to.'")
                .default_value("demo/example/**"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if args.is_present("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
    }

    let key_expr = KeyExpr::try_from(args.value_of("key").unwrap())
        .unwrap()
        .into_owned();

    (config, key_expr)
}
