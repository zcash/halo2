window.BENCHMARK_DATA = {
  "lastUpdate": 1664197899118,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "distinct": true,
          "id": "6f692c0e53c50f18aae09ca731d144d4bc9e951b",
          "message": "There is no z' in the lookup argument, only z.\n\nSigned-off-by: Daira Hopwood <daira@jacaranda.org>",
          "timestamp": "2022-09-26T20:14:08+08:00",
          "tree_id": "a78bc7d4167925eac8560b9b2f4856d04791b456",
          "url": "https://github.com/zcash/halo2/commit/6f692c0e53c50f18aae09ca731d144d4bc9e951b"
        },
        "date": 1664197893656,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60619087,
            "range": "± 3674099",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2984272,
            "range": "± 61216",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 131442786,
            "range": "± 3562479",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3567708,
            "range": "± 98353",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 183200802,
            "range": "± 3506362",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4005842,
            "range": "± 42382",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38911,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141712,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153774,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 251426,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 251409,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144614,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156871,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 254479,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 254444,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302656,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314883,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412336,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 412463,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3134353,
            "range": "± 2173",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5960291,
            "range": "± 17427",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10797751,
            "range": "± 47923",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22286955,
            "range": "± 245551",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41480833,
            "range": "± 304901",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 83307399,
            "range": "± 200763",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7804,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8804,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15238,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19114,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27879,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47155,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98486,
            "range": "± 7370",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 191903,
            "range": "± 8728",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 388375,
            "range": "± 19639",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 816444,
            "range": "± 43528",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1727676,
            "range": "± 44825",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3710607,
            "range": "± 88594",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8124354,
            "range": "± 251259",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17802025,
            "range": "± 255268",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 39022380,
            "range": "± 411954",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 88377972,
            "range": "± 3502936",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28427,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28509,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153060829,
            "range": "± 463196",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328024824,
            "range": "± 1002220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 706523521,
            "range": "± 3330042",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1518884422,
            "range": "± 18278071",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3263480109,
            "range": "± 3110307",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6965074008,
            "range": "± 14193495",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14820604511,
            "range": "± 85371858",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31496629996,
            "range": "± 29498308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66664874726,
            "range": "± 48684317",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95912468,
            "range": "± 522678",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 163804037,
            "range": "± 325890",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 291273343,
            "range": "± 1332981",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 530416088,
            "range": "± 2463027",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 986447596,
            "range": "± 3103203",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1863920834,
            "range": "± 5246241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3555095052,
            "range": "± 5881560",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6866344628,
            "range": "± 174603584",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13258284528,
            "range": "± 29414538",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5181500,
            "range": "± 41047",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7990959,
            "range": "± 56164",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12797623,
            "range": "± 479518",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21755103,
            "range": "± 547651",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37418629,
            "range": "± 304545",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67009838,
            "range": "± 482349",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120083964,
            "range": "± 2302413",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222086406,
            "range": "± 17172241",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 403710713,
            "range": "± 2151405",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}