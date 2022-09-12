window.BENCHMARK_DATA = {
  "lastUpdate": 1663008122803,
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
          "id": "5d653e5d4ebe6cc192366a6a524d1cce32520949",
          "message": "Merge pull request #649 from zcash/cleanups\n\nVarious cleanups",
          "timestamp": "2022-09-12T18:44:45+01:00",
          "tree_id": "ca6b89aab3663b534e59e5fdbdb6abda0072ae64",
          "url": "https://github.com/zcash/halo2/commit/5d653e5d4ebe6cc192366a6a524d1cce32520949"
        },
        "date": 1663008115701,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60798797,
            "range": "± 2623988",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3041627,
            "range": "± 337339",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 132270140,
            "range": "± 2960742",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3664675,
            "range": "± 56939",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 184495319,
            "range": "± 3150312",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4026497,
            "range": "± 284643",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38628,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141625,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153754,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 248143,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 248201,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144593,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156782,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 251272,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 251150,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 300309,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314670,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 409220,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 409139,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3041895,
            "range": "± 2224",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5908325,
            "range": "± 4977",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10662364,
            "range": "± 32410",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22772821,
            "range": "± 161546",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43485168,
            "range": "± 141167",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84551218,
            "range": "± 269635",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7739,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8959,
            "range": "± 1433",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16267,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20475,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27895,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47003,
            "range": "± 895",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 99267,
            "range": "± 5842",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 191578,
            "range": "± 8898",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 387674,
            "range": "± 13455",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 819250,
            "range": "± 22687",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1732932,
            "range": "± 78889",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3713172,
            "range": "± 62987",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8189309,
            "range": "± 139323",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17682525,
            "range": "± 323510",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38887338,
            "range": "± 646296",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 87031186,
            "range": "± 2461701",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28448,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28539,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153915582,
            "range": "± 5737257",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 329303740,
            "range": "± 1543315",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707301508,
            "range": "± 2279308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1524920506,
            "range": "± 7655200",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3262276814,
            "range": "± 6123706",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6973473257,
            "range": "± 14676333",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14854781367,
            "range": "± 15266383",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31564131326,
            "range": "± 235589301",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66723990860,
            "range": "± 58694850",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96121285,
            "range": "± 1754587",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 165954915,
            "range": "± 1588149",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 291664081,
            "range": "± 4456673",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 533066766,
            "range": "± 3743709",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 989708282,
            "range": "± 10689791",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1866390074,
            "range": "± 5588360",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3552110308,
            "range": "± 14023880",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6870759419,
            "range": "± 13909049",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13269914249,
            "range": "± 44619444",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5135421,
            "range": "± 93470",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8025169,
            "range": "± 219567",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12877004,
            "range": "± 654756",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21844547,
            "range": "± 614520",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37682490,
            "range": "± 438737",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67162956,
            "range": "± 710135",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120368433,
            "range": "± 1958376",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222445594,
            "range": "± 5738835",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404823745,
            "range": "± 9567449",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}