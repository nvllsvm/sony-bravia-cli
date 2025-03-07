use std::env;
use std::io::Write;
use std::time::Duration;

const CONTROL_REQUEST: u8 = 0x8c;
const QUERY_REQUEST: u8 = 0x83;
const CATEGORY: u8 = 0x00;
const POWER_FUNCTION: u8 = 0x00;
const INPUT_SELECT_FUNCTION: u8 = 0x02;
const VOLUME_CONTROL_FUNCTION: u8 = 0x05;
const PICTURE_FUNCTION: u8 = 0x0d;
const DISPLAY_FUNCTION: u8 = 0x0f;
const BRIGHTNESS_CONTROL_FUNCTION: u8 = 0x24;
const MUTING_FUNCTION: u8 = 0x06;

const INPUT_TYPE_HDMI: u8 = 0x04;

const RESPONSE_HEADER: u8 = 0x70;
const RESPONSE_ANSWER: u8 = 0x00;

fn checksum(command: &Vec<u8>) -> u8 {
    let s: u8 = command.iter().sum();
    return s % 255;
}

fn power_on(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, POWER_FUNCTION, 0x02, 0x01];
    write_command(port, args);
}

fn power_off(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, POWER_FUNCTION, 0x02, 0x00];
    write_command(port, args);
}

fn picture_toggle(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, PICTURE_FUNCTION, 0x02, 0x00];
    write_command(port, args);
}

fn picture_on(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, PICTURE_FUNCTION, 0x02, 0x01];
    write_command(port, args);
}

fn picture_off(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, PICTURE_FUNCTION, 0x02, 0x00];
    write_command(port, args);
}

fn display_toggle(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, DISPLAY_FUNCTION, 0x02, 0x00];
    write_command(port, args);
}

fn brightness_up(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        BRIGHTNESS_CONTROL_FUNCTION,
        0x03,
        0x00,
        0x00,
    ];
    write_command(port, args);
}

fn brightness_down(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        BRIGHTNESS_CONTROL_FUNCTION,
        0x03,
        0x00,
        0x01,
    ];
    write_command(port, args);
}

fn brightness_min(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        BRIGHTNESS_CONTROL_FUNCTION,
        0x03,
        0x01,
        0x00,
    ];
    write_command(port, args);
}

fn brightness_max(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        BRIGHTNESS_CONTROL_FUNCTION,
        0x03,
        0x01,
        0x32,
    ];
    write_command(port, args);
}

fn volume_up(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        VOLUME_CONTROL_FUNCTION,
        0x03,
        0x00,
        0x00,
    ];
    write_command(port, args);
}

fn volume_down(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        VOLUME_CONTROL_FUNCTION,
        0x03,
        0x00,
        0x01,
    ];
    write_command(port, args);
}

fn input_select(port: &mut Box<dyn serialport::SerialPort>, input_type: u8, input_num: u8) {
    let args = vec![
        CONTROL_REQUEST,
        CATEGORY,
        INPUT_SELECT_FUNCTION,
        0x03,
        input_type,
        input_num,
    ];
    write_command(port, args);
}

fn mute_toggle(port: &mut Box<dyn serialport::SerialPort>) {
    let args = vec![CONTROL_REQUEST, CATEGORY, MUTING_FUNCTION, 0x02, 0x00];
    write_command(port, args);
}

fn is_powered_on(port: &mut Box<dyn serialport::SerialPort>) -> bool {
    let args = vec![QUERY_REQUEST, CATEGORY, POWER_FUNCTION, 0xff, 0xff];
    let data = write_command(port, args);
    return data[0] == 1;
}

fn power_toggle(port: &mut Box<dyn serialport::SerialPort>) {
    if is_powered_on(port) {
        println!("is on - turning off!");
        power_off(port);
    } else {
        println!("is off - turning on!");
        power_on(port);
    }
}

fn print_status(port: &mut Box<dyn serialport::SerialPort>) {
    if is_powered_on(port) {
        println!("Power: on");
    } else {
        println!("Power: off");
    }
}

fn print_usage() {
    eprintln!("usage: DEVICE [on|off|power|volume-up|volume-down|mute|status]");
}

fn write_command(port: &mut Box<dyn serialport::SerialPort>, contents: Vec<u8>) -> Vec<u8> {
    let mut vec = contents.clone();
    let c = checksum(&vec);
    vec.push(c);
    port.write_all(&vec).unwrap();

    let mut resp_buf = vec![0; 3];
    port.read(resp_buf.as_mut_slice())
        .expect("failure to read response");

    if resp_buf[0] != RESPONSE_HEADER {
        eprintln!("error: unexpected response header");
        std::process::exit(1);
    }
    if resp_buf[1] != RESPONSE_ANSWER {
        eprintln!("error: unexpected response answer");
        std::process::exit(1);
    }
    if vec[0] == QUERY_REQUEST {
        let mut resp_data_buf = vec![0; resp_buf[2] as usize];
        port.read(resp_data_buf.as_mut_slice())
            .expect("failure to read response data");
        let resp_checksum = resp_data_buf.pop().expect("error");
        resp_buf.extend(resp_data_buf.clone());
        if resp_checksum != checksum(&resp_buf) {
            eprintln!("error: invalid response checksum");
            std::process::exit(1);
        }
        return resp_data_buf;
    } else {
        let resp_checksum = resp_buf.pop().expect("error");
        if resp_checksum != checksum(&resp_buf) {
            eprintln!("error: invalid response checksum");
            std::process::exit(1);
        }
        return vec![0; 0];
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {}
        _ => {
            print_usage();
            eprintln!("error: unexpected argument(s)");
            std::process::exit(1);
        }
    }

    let mut port = serialport::new(&args[1], 9600)
        .timeout(Duration::from_millis(500))
        .open()
        .expect("Failed to open port.");
    match &args[2][..] {
        "on" => power_on(&mut port),
        "off" => power_off(&mut port),
        "power" => power_toggle(&mut port),
        "picture" => picture_toggle(&mut port),
        "picture-on" => picture_on(&mut port),
        "picture-off" => picture_off(&mut port),
        "display" => display_toggle(&mut port),
        "brightness-up" => brightness_up(&mut port),
        "brightness-down" => brightness_down(&mut port),
        "brightness-min" => brightness_min(&mut port),
        "brightness-max" => brightness_max(&mut port),
        "volume-up" => volume_up(&mut port),
        "volume-down" => volume_down(&mut port),
        "mute" => mute_toggle(&mut port),
        "input-hdmi-1" => input_select(&mut port, INPUT_TYPE_HDMI, 1),
        "input-hdmi-2" => input_select(&mut port, INPUT_TYPE_HDMI, 2),
        "input-hdmi-3" => input_select(&mut port, INPUT_TYPE_HDMI, 3),
        "input-hdmi-4" => input_select(&mut port, INPUT_TYPE_HDMI, 4),
        "input-hdmi-5" => input_select(&mut port, INPUT_TYPE_HDMI, 5),
        "status" => print_status(&mut port),
        _ => {
            eprintln!("error: invalid action");
            std::process::exit(1);
        }
    };
}
