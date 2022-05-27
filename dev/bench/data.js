window.BENCHMARK_DATA = {
  "lastUpdate": 1653670226191,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c0db68aa0535cf4071ee58f5b72c8211f888b5a3",
          "message": "Merge pull request #589 from zcash/protocol-rule-links\n\nhalo2_gadgets: Add protocol rule links for the chip constraints",
          "timestamp": "2022-05-27T16:46:56+01:00",
          "tree_id": "8d0b90a255c5518a64b08dd5d78dc4324035c182",
          "url": "https://github.com/zcash/halo2/commit/c0db68aa0535cf4071ee58f5b72c8211f888b5a3"
        },
        "date": 1653670220201,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 78908054,
            "range": "± 717840",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3822562,
            "range": "± 45505",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 179461902,
            "range": "± 1185334",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5750393,
            "range": "± 42426",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 250043422,
            "range": "± 739810",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6655734,
            "range": "± 53890",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 42533,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164689,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 180089,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 290872,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 290877,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168239,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183585,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 294268,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 294285,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 352160,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 367238,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 477746,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 477764,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3357130,
            "range": "± 1997",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5871303,
            "range": "± 4092",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10472532,
            "range": "± 8872",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 20838483,
            "range": "± 44412",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 38901047,
            "range": "± 220984",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 77214618,
            "range": "± 491368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6745,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 7892,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15010,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18969,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28296,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 49914,
            "range": "± 1278",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 101962,
            "range": "± 5896",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 199148,
            "range": "± 8465",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 408134,
            "range": "± 9272",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 852867,
            "range": "± 10826",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1802914,
            "range": "± 16958",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3839560,
            "range": "± 30364",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8160724,
            "range": "± 286454",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17372776,
            "range": "± 177301",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37184864,
            "range": "± 404013",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 81564519,
            "range": "± 826092",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34965,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35214,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 177753752,
            "range": "± 295572",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 382827559,
            "range": "± 745251",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 826285494,
            "range": "± 2745292",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1776307265,
            "range": "± 11149570",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3808699356,
            "range": "± 65821043",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8135037973,
            "range": "± 13230583",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17323030935,
            "range": "± 34619071",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36812536561,
            "range": "± 19047105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77831128203,
            "range": "± 50134638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 109264634,
            "range": "± 770300",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 187467298,
            "range": "± 328623",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 332425179,
            "range": "± 1609648",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 606059891,
            "range": "± 1580446",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1129134326,
            "range": "± 13047091",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2126376291,
            "range": "± 4767246",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4054516184,
            "range": "± 7972803",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7805963457,
            "range": "± 64961301",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15014881285,
            "range": "± 29584155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5933652,
            "range": "± 77361",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9167218,
            "range": "± 74189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14690069,
            "range": "± 88482",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25167800,
            "range": "± 246181",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42992392,
            "range": "± 168925",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76402870,
            "range": "± 713094",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137319022,
            "range": "± 837490",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 252649704,
            "range": "± 1432248",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 459206230,
            "range": "± 4676069",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}