window.BENCHMARK_DATA = {
  "lastUpdate": 1662135953164,
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1806b88c7455f687e486ff23ae5e147b16d540a4",
          "message": "Merge pull request #626 from daira/daira-book-fullwidth\n\n[book] Full-width variable-base scalar mul allowing the scalar to be outside the base field",
          "timestamp": "2022-09-02T16:26:59+01:00",
          "tree_id": "e0c34324855541aa903f233047b677d04be724c1",
          "url": "https://github.com/zcash/halo2/commit/1806b88c7455f687e486ff23ae5e147b16d540a4"
        },
        "date": 1662135945995,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60879834,
            "range": "± 5318458",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2977072,
            "range": "± 52911",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 133152061,
            "range": "± 2700338",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3708547,
            "range": "± 134805",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 185396562,
            "range": "± 3759877",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4022897,
            "range": "± 40613",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38848,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141456,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153613,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 251494,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 251523,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144390,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156519,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 254484,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 254369,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302070,
            "range": "± 842",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314211,
            "range": "± 218",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412066,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 411955,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3056175,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5883275,
            "range": "± 3078",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10191159,
            "range": "± 5426",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22096654,
            "range": "± 252574",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43224800,
            "range": "± 750957",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 83754836,
            "range": "± 309389",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7764,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8862,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15840,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19682,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27907,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46872,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97441,
            "range": "± 8007",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188917,
            "range": "± 9001",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 389571,
            "range": "± 12998",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 820162,
            "range": "± 58657",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1734877,
            "range": "± 41650",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3694720,
            "range": "± 34027",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8135929,
            "range": "± 66041",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17717657,
            "range": "± 1584959",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 39112531,
            "range": "± 546810",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 90986070,
            "range": "± 1774226",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28399,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28519,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153501289,
            "range": "± 1019500",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327686716,
            "range": "± 885340",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 708495878,
            "range": "± 9475967",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1519594138,
            "range": "± 3104170",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3266232650,
            "range": "± 9996523",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6954845783,
            "range": "± 9646387",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14823355774,
            "range": "± 24201891",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31553129946,
            "range": "± 44918952",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66797240025,
            "range": "± 128956342",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96529261,
            "range": "± 3385578",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166152227,
            "range": "± 521875",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 294222646,
            "range": "± 5861338",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 537301890,
            "range": "± 1768427",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 998899321,
            "range": "± 3321624",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1899001909,
            "range": "± 13458041",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3608656914,
            "range": "± 14483256",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6967910101,
            "range": "± 14312290",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13409539526,
            "range": "± 35872148",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5147099,
            "range": "± 39379",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8049240,
            "range": "± 56291",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12859383,
            "range": "± 398123",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21923558,
            "range": "± 851449",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37664953,
            "range": "± 302976",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67249700,
            "range": "± 425407",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120576570,
            "range": "± 1469136",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222216705,
            "range": "± 2670415",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404729677,
            "range": "± 2797878",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}