extern crate bitsparrow;

use bitsparrow::{Encoder, Decoder};

#[test]
fn eat_own_dog_food() {
  const PI: f64 = 3.141592653589793;
  let expected = vec![
    0xc8,0x23,0x29,0x49,0x96,0x02,0xd2,0xd6,0x8a,0xd0,0xb6,0x69,0xfd,
    0x2e,0x0f,0x42,0x69,0x74,0x53,0x70,0x61,0x72,0x72,0x6f,0x77,0x20,
    0xf0,0x9f,0x90,0xa6,0x83,0x69,0x53,0x70,0x61,0x72,0x72,0x6f,0x77,
    0x20,0x2f,0xcb,0x88,0x73,0x70,0x65,0x72,0x2e,0x6f,0xca,0x8a,0x2f,
    0x0a,0x0a,0x55,0x6e,0x64,0x65,0x72,0x20,0x74,0x68,0x65,0x20,0x63,
    0x6c,0x61,0x73,0x73,0x69,0x66,0x69,0x63,0x61,0x74,0x69,0x6f,0x6e,
    0x20,0x75,0x73,0x65,0x64,0x20,0x69,0x6e,0x20,0x74,0x68,0x65,0x20,
    0x48,0x61,0x6e,0x64,0x62,0x6f,0x6f,0x6b,0x20,0x6f,0x66,0x20,0x74,
    0x68,0x65,0x20,0x42,0x69,0x72,0x64,0x73,0x20,0x6f,0x66,0x20,0x74,
    0x68,0x65,0x20,0x57,0x6f,0x72,0x6c,0x64,0x20,0x28,0x48,0x42,0x57,
    0x29,0x20,0x6d,0x61,0x69,0x6e,0x20,0x67,0x72,0x6f,0x75,0x70,0x69,
    0x6e,0x67,0x73,0x20,0x6f,0x66,0x20,0x74,0x68,0x65,0x20,0x73,0x70,
    0x61,0x72,0x72,0x6f,0x77,0x73,0x20,0x61,0x72,0x65,0x20,0x74,0x68,
    0x65,0x20,0x74,0x72,0x75,0x65,0x20,0x73,0x70,0x61,0x72,0x72,0x6f,
    0x77,0x73,0x20,0x28,0x67,0x65,0x6e,0x75,0x73,0x20,0x50,0x61,0x73,
    0x73,0x65,0x72,0x29,0x2c,0x20,0x74,0x68,0x65,0x20,0x73,0x6e,0x6f,
    0x77,0x66,0x69,0x6e,0x63,0x68,0x65,0x73,0x20,0x28,0x74,0x79,0x70,
    0x69,0x63,0x61,0x6c,0x6c,0x79,0x20,0x6f,0x6e,0x65,0x20,0x67,0x65,
    0x6e,0x75,0x73,0x2c,0x20,0x4d,0x6f,0x6e,0x74,0x69,0x66,0x72,0x69,
    0x6e,0x67,0x69,0x6c,0x6c,0x61,0x29,0x2c,0x20,0x61,0x6e,0x64,0x20,
    0x74,0x68,0x65,0x20,0x72,0x6f,0x63,0x6b,0x20,0x73,0x70,0x61,0x72,
    0x72,0x6f,0x77,0x73,0x20,0x28,0x50,0x65,0x74,0x72,0x6f,0x6e,0x69,
    0x61,0x20,0x61,0x6e,0x64,0x20,0x74,0x68,0x65,0x20,0x70,0x61,0x6c,
    0x65,0x20,0x72,0x6f,0x63,0x6b,0x66,0x69,0x6e,0x63,0x68,0x29,0x2e,
    0x20,0x54,0x68,0x65,0x73,0x65,0x20,0x67,0x72,0x6f,0x75,0x70,0x73,
    0x20,0x61,0x72,0x65,0x20,0x73,0x69,0x6d,0x69,0x6c,0x61,0x72,0x20,
    0x74,0x6f,0x20,0x65,0x61,0x63,0x68,0x20,0x6f,0x74,0x68,0x65,0x72,
    0x2c,0x20,0x61,0x6e,0x64,0x20,0x61,0x72,0x65,0x20,0x65,0x61,0x63,
    0x68,0x20,0x66,0x61,0x69,0x72,0x6c,0x79,0x20,0x68,0x6f,0x6d,0x6f,
    0x67,0x65,0x6e,0x65,0x6f,0x75,0x73,0x2c,0x20,0x65,0x73,0x70,0x65,
    0x63,0x69,0x61,0x6c,0x6c,0x79,0x20,0x50,0x61,0x73,0x73,0x65,0x72,
    0x2e,0x5b,0x34,0x5d,0x20,0x53,0x6f,0x6d,0x65,0x20,0x63,0x6c,0x61,
    0x73,0x73,0x69,0x66,0x69,0x63,0x61,0x74,0x69,0x6f,0x6e,0x73,0x20,
    0x61,0x6c,0x73,0x6f,0x20,0x69,0x6e,0x63,0x6c,0x75,0x64,0x65,0x20,
    0x74,0x68,0x65,0x20,0x73,0x70,0x61,0x72,0x72,0x6f,0x77,0x2d,0x77,
    0x65,0x61,0x76,0x65,0x72,0x73,0x20,0x28,0x50,0x6c,0x6f,0x63,0x65,
    0x70,0x61,0x73,0x73,0x65,0x72,0x29,0x20,0x61,0x6e,0x64,0x20,0x73,
    0x65,0x76,0x65,0x72,0x61,0x6c,0x20,0x6f,0x74,0x68,0x65,0x72,0x20,
    0x41,0x66,0x72,0x69,0x63,0x61,0x6e,0x20,0x67,0x65,0x6e,0x65,0x72,
    0x61,0x20,0x28,0x6f,0x74,0x68,0x65,0x72,0x77,0x69,0x73,0x65,0x20,
    0x63,0x6c,0x61,0x73,0x73,0x69,0x66,0x69,0x65,0x64,0x20,0x61,0x6d,
    0x6f,0x6e,0x67,0x20,0x74,0x68,0x65,0x20,0x77,0x65,0x61,0x76,0x65,
    0x72,0x73,0x2c,0x20,0x50,0x6c,0x6f,0x63,0x65,0x69,0x64,0x61,0x65,
    0x29,0x5b,0x34,0x5d,0x20,0x77,0x68,0x69,0x63,0x68,0x20,0x61,0x72,
    0x65,0x20,0x6d,0x6f,0x72,0x70,0x68,0x6f,0x6c,0x6f,0x67,0x69,0x63,
    0x61,0x6c,0x6c,0x79,0x20,0x73,0x69,0x6d,0x69,0x6c,0x61,0x72,0x20,
    0x74,0x6f,0x20,0x50,0x61,0x73,0x73,0x65,0x72,0x2e,0x5b,0x35,0x5d,
    0x20,0x41,0x63,0x63,0x6f,0x72,0x64,0x69,0x6e,0x67,0x20,0x74,0x6f,
    0x20,0x61,0x20,0x73,0x74,0x75,0x64,0x79,0x20,0x6f,0x66,0x20,0x6d,
    0x6f,0x6c,0x65,0x63,0x75,0x6c,0x61,0x72,0x20,0x61,0x6e,0x64,0x20,
    0x73,0x6b,0x65,0x6c,0x65,0x74,0x61,0x6c,0x20,0x65,0x76,0x69,0x64,
    0x65,0x6e,0x63,0x65,0x20,0x62,0x79,0x20,0x4a,0x6f,0x6e,0x20,0x46,
    0x6a,0x65,0x6c,0x64,0x73,0xc3,0xa5,0x20,0x61,0x6e,0x64,0x20,0x63,
    0x6f,0x6c,0x6c,0x65,0x61,0x67,0x75,0x65,0x73,0x2c,0x20,0x74,0x68,
    0x65,0x20,0x63,0x69,0x6e,0x6e,0x61,0x6d,0x6f,0x6e,0x20,0x69,0x62,
    0x6f,0x6e,0x20,0x6f,0x66,0x20,0x74,0x68,0x65,0x20,0x50,0x68,0x69,
    0x6c,0x69,0x70,0x70,0x69,0x6e,0x65,0x73,0x2c,0x20,0x70,0x72,0x65,
    0x76,0x69,0x6f,0x75,0x73,0x6c,0x79,0x20,0x63,0x6f,0x6e,0x73,0x69,
    0x64,0x65,0x72,0x65,0x64,0x20,0x74,0x6f,0x20,0x62,0x65,0x20,0x61,
    0x20,0x77,0x68,0x69,0x74,0x65,0x2d,0x65,0x79,0x65,0x2c,0x20,0x69,
    0x73,0x20,0x61,0x20,0x73,0x69,0x73,0x74,0x65,0x72,0x20,0x74,0x61,
    0x78,0x6f,0x6e,0x20,0x74,0x6f,0x20,0x74,0x68,0x65,0x20,0x73,0x70,
    0x61,0x72,0x72,0x6f,0x77,0x73,0x20,0x61,0x73,0x20,0x64,0x65,0x66,
    0x69,0x6e,0x65,0x64,0x20,0x62,0x79,0x20,0x74,0x68,0x65,0x20,0x48,
    0x42,0x57,0x2e,0x20,0x54,0x68,0x65,0x79,0x20,0x74,0x68,0x65,0x72,
    0x65,0x66,0x6f,0x72,0x65,0x20,0x63,0x6c,0x61,0x73,0x73,0x69,0x66,
    0x79,0x20,0x69,0x74,0x20,0x61,0x73,0x20,0x69,0x74,0x73,0x20,0x6f,
    0x77,0x6e,0x20,0x73,0x75,0x62,0x66,0x61,0x6d,0x69,0x6c,0x79,0x20,
    0x77,0x69,0x74,0x68,0x69,0x6e,0x20,0x50,0x61,0x73,0x73,0x65,0x72,
    0x69,0x64,0x61,0x65,0x2e,0x5b,0x35,0x5d,0x06,0x01,0x02,0x03,0x04,
    0x05,0x06,0x64,0xa7,0x10,0xc0,0x0f,0x42,0x40,0xff,0xff,0xff,0xff,
    0x40,0x49,0x0f,0xdb,0x40,0x09,0x21,0xfb,0x54,0x44,0x2d,0x18
  ];
  let long_text = "Sparrow /ˈsper.oʊ/\n\nUnder the classification used in the Handbook of the Birds of the World (HBW) main groupings of the sparrows are the true sparrows (genus Passer), the snowfinches (typically one genus, Montifringilla), and the rock sparrows (Petronia and the pale rockfinch). These groups are similar to each other, and are each fairly homogeneous, especially Passer.[4] Some classifications also include the sparrow-weavers (Plocepasser) and several other African genera (otherwise classified among the weavers, Ploceidae)[4] which are morphologically similar to Passer.[5] According to a study of molecular and skeletal evidence by Jon Fjeldså and colleagues, the cinnamon ibon of the Philippines, previously considered to be a white-eye, is a sister taxon to the sparrows as defined by the HBW. They therefore classify it as its own subfamily within Passeridae.[5]";
  let blob = [1,2,3,4,5,6];

  let buffer = Encoder::new()
    .uint8(200)
    .uint16(9001)
    .uint32(1234567890)
    .int8(-42)
    .int16(-30000)
    .int32(-1234567890)
    .string("BitSparrow 🐦")
    .string(long_text)
    .blob(&blob)
    .size(100)
    .size(10000)
    .size(1000000)
    .size(1073741823)
    .float32(PI as f32)
    .float64(PI)
    .encode()
    .unwrap();

  assert_eq!(buffer, expected);

  let mut decoder = Decoder::new(&buffer);
  assert_eq!(decoder.uint8().unwrap(), 200);
  assert_eq!(decoder.uint16().unwrap(), 9001);
  assert_eq!(decoder.uint32().unwrap(), 1234567890);
  assert_eq!(decoder.int8().unwrap(), -42);
  assert_eq!(decoder.int16().unwrap(), -30000);
  assert_eq!(decoder.int32().unwrap(), -1234567890);
  assert_eq!(decoder.string().unwrap(), "BitSparrow 🐦");
  assert_eq!(decoder.string().unwrap(), long_text);
  assert_eq!(decoder.blob().unwrap(), blob);
  assert_eq!(decoder.size().unwrap(), 100);
  assert_eq!(decoder.size().unwrap(), 10000);
  assert_eq!(decoder.size().unwrap(), 1000000);
  assert_eq!(decoder.size().unwrap(), 1073741823);
  assert_eq!(decoder.float32().unwrap(), PI as f32);
  assert_eq!(decoder.float64().unwrap(), PI);
}

#[test]
fn stacking_bools() {
  let buffer = Encoder::new()
    .bool(true)
    .bool(false)
    .bool(true)
    .bool(false)
    .bool(false)
    .bool(false)
    .bool(true)
    .bool(true)
    .bool(false)
    .uint8(10)
    .bool(true)
    .encode()
    .unwrap();

  assert_eq!(buffer.len(), 4);

  let mut decoder = Decoder::new(&buffer);
  assert_eq!(decoder.bool().unwrap(), true);
  assert_eq!(decoder.bool().unwrap(), false);
  assert_eq!(decoder.bool().unwrap(), true);
  assert_eq!(decoder.bool().unwrap(), false);
  assert_eq!(decoder.bool().unwrap(), false);
  assert_eq!(decoder.bool().unwrap(), false);
  assert_eq!(decoder.bool().unwrap(), true);
  assert_eq!(decoder.bool().unwrap(), true);
  assert_eq!(decoder.bool().unwrap(), false);
  assert_eq!(decoder.uint8().unwrap(), 10);
  assert_eq!(decoder.bool().unwrap(), true);
}
