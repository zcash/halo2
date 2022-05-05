window.BENCHMARK_DATA = {
  "lastUpdate": 1651784984513,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "taylor@defuse.ca",
            "name": "Taylor Hornby",
            "username": "defuse"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "64e7efb0d4c76f1711b7d77ca40e6d3b9d7de528",
          "message": "Merge pull request #542 from zcash/relicense-mit-or-apache-2.0\n\nRelicense Halo 2 crates as MIT OR Apache 2.0",
          "timestamp": "2022-05-05T13:50:45-06:00",
          "tree_id": "2bd46320f85ef5f14ecc3dc23dfdf1023cfb6f64",
          "url": "https://github.com/zcash/halo2/commit/64e7efb0d4c76f1711b7d77ca40e6d3b9d7de528"
        },
        "date": 1651784979491,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 96563804,
            "range": "± 7507303",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4327907,
            "range": "± 409251",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 228546195,
            "range": "± 11987818",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6471316,
            "range": "± 520513",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 309112498,
            "range": "± 9661952",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7805196,
            "range": "± 719348",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47440,
            "range": "± 3393",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 181995,
            "range": "± 13143",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 197208,
            "range": "± 18679",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 313910,
            "range": "± 22512",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 326974,
            "range": "± 26032",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 182447,
            "range": "± 7303",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 196773,
            "range": "± 9036",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 319855,
            "range": "± 17074",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 318691,
            "range": "± 16571",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 380871,
            "range": "± 23179",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 397140,
            "range": "± 25367",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 511812,
            "range": "± 19964",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 516139,
            "range": "± 20478",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3807971,
            "range": "± 185579",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7731220,
            "range": "± 347510",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 14147846,
            "range": "± 460908",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 30001699,
            "range": "± 575396",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 55491771,
            "range": "± 1594425",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 109186848,
            "range": "± 2527888",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 18761,
            "range": "± 1717",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 21676,
            "range": "± 1662",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 27458,
            "range": "± 1641",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 38124,
            "range": "± 5856",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 58824,
            "range": "± 5654",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 104450,
            "range": "± 9519",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 210133,
            "range": "± 25221",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 414955,
            "range": "± 44640",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 862811,
            "range": "± 65524",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1823638,
            "range": "± 137477",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 3763891,
            "range": "± 458554",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 8126082,
            "range": "± 637477",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 17253844,
            "range": "± 1235761",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 37079275,
            "range": "± 2555388",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 79800546,
            "range": "± 4530981",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 166668176,
            "range": "± 7487795",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 35406,
            "range": "± 3624",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35166,
            "range": "± 2144",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 254582785,
            "range": "± 12503660",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 543785524,
            "range": "± 29279481",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 1122863443,
            "range": "± 13941119",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 2391046519,
            "range": "± 25093862",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 5022144181,
            "range": "± 48851935",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 10682417120,
            "range": "± 81527701",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 22623062211,
            "range": "± 122438022",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 47397234675,
            "range": "± 217299655",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 100166107457,
            "range": "± 1050366477",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 130765821,
            "range": "± 2146984",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 228416197,
            "range": "± 4134494",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 400048880,
            "range": "± 9900063",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 717899943,
            "range": "± 15843865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1371924315,
            "range": "± 17945055",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2574269505,
            "range": "± 38237780",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4892136918,
            "range": "± 79536337",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9371921900,
            "range": "± 89086487",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17943589713,
            "range": "± 159645042",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6739292,
            "range": "± 564751",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10327968,
            "range": "± 588852",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16589951,
            "range": "± 931406",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27884480,
            "range": "± 1958302",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 48483979,
            "range": "± 3993329",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 85545097,
            "range": "± 4284486",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 154926204,
            "range": "± 8239711",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 291515865,
            "range": "± 8793320",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 534192017,
            "range": "± 36269639",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}