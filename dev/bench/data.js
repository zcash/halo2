window.BENCHMARK_DATA = {
  "lastUpdate": 1652456250523,
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
          "id": "8075b216e850035cde940c96eb93bf285254ee2e",
          "message": "Merge pull request #584 from trel/remove_dsstore\n\nremove .DS_Store",
          "timestamp": "2022-05-13T15:29:23+01:00",
          "tree_id": "2ca8f65dba2529e203e102a07f99bb6ef2fc0a66",
          "url": "https://github.com/zcash/halo2/commit/8075b216e850035cde940c96eb93bf285254ee2e"
        },
        "date": 1652456245070,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 91018073,
            "range": "± 1611201",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3987340,
            "range": "± 89819",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 209165538,
            "range": "± 4342175",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5881735,
            "range": "± 195063",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 286149323,
            "range": "± 4765864",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7086855,
            "range": "± 101025",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 42770,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 167929,
            "range": "± 2798",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 175154,
            "range": "± 4807",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 288602,
            "range": "± 8211",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 279973,
            "range": "± 10049",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 165021,
            "range": "± 8391",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 184053,
            "range": "± 4957",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 299982,
            "range": "± 9133",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 299670,
            "range": "± 6505",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 343810,
            "range": "± 13843",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 379056,
            "range": "± 4230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 489018,
            "range": "± 8977",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 484442,
            "range": "± 12677",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3744771,
            "range": "± 32630",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6952292,
            "range": "± 45083",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 11941978,
            "range": "± 104456",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27695376,
            "range": "± 340715",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 50145060,
            "range": "± 358585",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 94480419,
            "range": "± 936004",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8469,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9863,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18354,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22457,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32672,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 55357,
            "range": "± 1622",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 114934,
            "range": "± 9541",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 225484,
            "range": "± 15717",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 455534,
            "range": "± 28652",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 982423,
            "range": "± 75638",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2085261,
            "range": "± 86320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4577139,
            "range": "± 215923",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9774689,
            "range": "± 332368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 20977324,
            "range": "± 703575",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 46592343,
            "range": "± 1386427",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 106422726,
            "range": "± 3205864",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33892,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34032,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 186753443,
            "range": "± 12722108",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 388155972,
            "range": "± 2445614",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 835080197,
            "range": "± 11035962",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1814491497,
            "range": "± 15171968",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3872828634,
            "range": "± 29510974",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8292414511,
            "range": "± 34425630",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17668488399,
            "range": "± 109179213",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37385251691,
            "range": "± 357751708",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 79245623203,
            "range": "± 555664236",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 116249936,
            "range": "± 2509049",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 196898807,
            "range": "± 1726977",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 352562287,
            "range": "± 1969113",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 635678771,
            "range": "± 11322652",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1200765313,
            "range": "± 7779947",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2273478567,
            "range": "± 28438186",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4331904514,
            "range": "± 307415308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8306353933,
            "range": "± 35889717",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16045474847,
            "range": "± 91702568",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6224710,
            "range": "± 248520",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9358933,
            "range": "± 212190",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15546425,
            "range": "± 168891",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26130541,
            "range": "± 498659",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 45352365,
            "range": "± 1727561",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 79157700,
            "range": "± 1933118",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 144564554,
            "range": "± 1216519",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 267073816,
            "range": "± 4479963",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 486409242,
            "range": "± 4513317",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}