window.BENCHMARK_DATA = {
  "lastUpdate": 1676064827632,
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
          "id": "e0f6512165d0bb76b05d41460d2cd4103fe3afe0",
          "message": "Merge pull request #731 from zcash/msrv-1.60\n\nBump MSRV to 1.60",
          "timestamp": "2023-02-10T20:36:20Z",
          "tree_id": "1a753c230b4932204d7f1834b4b306c345128457",
          "url": "https://github.com/zcash/halo2/commit/e0f6512165d0bb76b05d41460d2cd4103fe3afe0"
        },
        "date": 1676064821556,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 92953283,
            "range": "± 9507819",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4615777,
            "range": "± 450821",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 168389900,
            "range": "± 4010747",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5549168,
            "range": "± 526388",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 227648986,
            "range": "± 6041153",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6921566,
            "range": "± 631814",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 35882,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141231,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153561,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 245893,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 245856,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144176,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156522,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 248774,
            "range": "± 1309",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 248772,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 301752,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314127,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 406496,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 406462,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2990197,
            "range": "± 7021",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5890899,
            "range": "± 6279",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10193055,
            "range": "± 22788",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21379636,
            "range": "± 153413",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41772545,
            "range": "± 234524",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82664674,
            "range": "± 146200",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7604,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8863,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15388,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20248,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 25950,
            "range": "± 2274",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 42292,
            "range": "± 6627",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 100316,
            "range": "± 25301",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 210774,
            "range": "± 37509",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 431852,
            "range": "± 64182",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 855105,
            "range": "± 89194",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1589295,
            "range": "± 163588",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3171981,
            "range": "± 320339",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 6929219,
            "range": "± 533758",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15013033,
            "range": "± 641463",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 32991023,
            "range": "± 490413",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 76803794,
            "range": "± 1100505",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28548,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28717,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 152857885,
            "range": "± 4340726",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 326962283,
            "range": "± 1706271",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 709438234,
            "range": "± 11274948",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1513290126,
            "range": "± 7017513",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3238948379,
            "range": "± 7223545",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6905339799,
            "range": "± 42892137",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14748493226,
            "range": "± 35096314",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31286004227,
            "range": "± 28233716",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66103714598,
            "range": "± 113105024",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 100179962,
            "range": "± 3820901",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 180102933,
            "range": "± 6029806",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 309499764,
            "range": "± 8664671",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 554319622,
            "range": "± 9683203",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1003003795,
            "range": "± 9570621",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1881191469,
            "range": "± 11645792",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3572519918,
            "range": "± 17223772",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6894925135,
            "range": "± 23936321",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13271436021,
            "range": "± 94546379",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5107424,
            "range": "± 48547",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7911050,
            "range": "± 857072",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12664270,
            "range": "± 393701",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21590484,
            "range": "± 1488855",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37054131,
            "range": "± 2167508",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 65859911,
            "range": "± 4108931",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 118569535,
            "range": "± 6796814",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 220713737,
            "range": "± 13987929",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 408769125,
            "range": "± 6512676",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}