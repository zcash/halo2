window.BENCHMARK_DATA = {
  "lastUpdate": 1660871574290,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "committer": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "distinct": true,
          "id": "5cd5e140f4631556d2fc666b210058b74179e635",
          "message": "[book] var-base-scalar-mul: Add missing closing parantheses",
          "timestamp": "2022-08-18T17:00:07-07:00",
          "tree_id": "92932691d1fba91383ac48bc25d9d36da1ae8531",
          "url": "https://github.com/zcash/halo2/commit/5cd5e140f4631556d2fc666b210058b74179e635"
        },
        "date": 1660871566389,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 76014053,
            "range": "± 7052746",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4194245,
            "range": "± 357427",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 165878661,
            "range": "± 5300797",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4790466,
            "range": "± 183204",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 224335206,
            "range": "± 7057106",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5268052,
            "range": "± 287367",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 53067,
            "range": "± 2786",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 175076,
            "range": "± 10058",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 187411,
            "range": "± 11810",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 324427,
            "range": "± 19800",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 315987,
            "range": "± 14398",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 180043,
            "range": "± 11743",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 200803,
            "range": "± 22462",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 330456,
            "range": "± 13657",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 317479,
            "range": "± 15655",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 380532,
            "range": "± 34436",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 381182,
            "range": "± 17866",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 517683,
            "range": "± 33682",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 523863,
            "range": "± 22354",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3947330,
            "range": "± 216352",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7313210,
            "range": "± 159345",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12877089,
            "range": "± 405033",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27333681,
            "range": "± 801931",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 52077612,
            "range": "± 2176062",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 103890130,
            "range": "± 1578253",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8081,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9479,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17100,
            "range": "± 719",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21632,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33084,
            "range": "± 3216",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 57955,
            "range": "± 3379",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 126430,
            "range": "± 20777",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 252358,
            "range": "± 21538",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 504962,
            "range": "± 58710",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1097614,
            "range": "± 106270",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2287949,
            "range": "± 119050",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 5016132,
            "range": "± 328777",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10769263,
            "range": "± 585294",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22799668,
            "range": "± 914566",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 50962649,
            "range": "± 2271592",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 110627702,
            "range": "± 4686113",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 36269,
            "range": "± 1604",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34905,
            "range": "± 3404",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 199418732,
            "range": "± 2993897",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 427372465,
            "range": "± 6847035",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 925496507,
            "range": "± 17293014",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1990681363,
            "range": "± 29762476",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4309047041,
            "range": "± 71003240",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 9212407270,
            "range": "± 104132821",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 19546019077,
            "range": "± 245080377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 41441954867,
            "range": "± 483789008",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 88597629960,
            "range": "± 727212873",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 126393963,
            "range": "± 1979750",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 219587336,
            "range": "± 5739419",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 393234080,
            "range": "± 6503807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 714521128,
            "range": "± 13444888",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1319697453,
            "range": "± 13987404",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2517465134,
            "range": "± 20526189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4813966759,
            "range": "± 89402470",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 9302719146,
            "range": "± 251526163",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 17738668557,
            "range": "± 188770865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6792986,
            "range": "± 332243",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10522138,
            "range": "± 519211",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 17447113,
            "range": "± 724264",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 29254777,
            "range": "± 1474156",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 50030194,
            "range": "± 3259560",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 89138178,
            "range": "± 3694451",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 161752380,
            "range": "± 5754375",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 295392016,
            "range": "± 10807539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 550533064,
            "range": "± 10055245",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}