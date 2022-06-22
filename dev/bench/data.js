window.BENCHMARK_DATA = {
  "lastUpdate": 1655920162042,
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
          "id": "f6efecc596813860e75cd4d0aa9b8c279c61a29c",
          "message": "Merge pull request #603 from zcash/batch-verifier-return-rng\n\nAdd `BatchVerifier::finalize_and_return_rng`",
          "timestamp": "2022-06-22T17:43:50+01:00",
          "tree_id": "cd512349c5cff8881a6d84a07ac3421ef7b45295",
          "url": "https://github.com/zcash/halo2/commit/f6efecc596813860e75cd4d0aa9b8c279c61a29c"
        },
        "date": 1655920154820,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 79865010,
            "range": "± 1952360",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3873679,
            "range": "± 23478",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 182686180,
            "range": "± 4012871",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5785719,
            "range": "± 102979",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 253400813,
            "range": "± 1683130",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6774720,
            "range": "± 62623",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 47193,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 163153,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 178404,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 291563,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 291843,
            "range": "± 1669",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 166592,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 181923,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 295149,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 294871,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 348425,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 363838,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 477051,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 476946,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3658751,
            "range": "± 2485",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6040002,
            "range": "± 2251",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10493148,
            "range": "± 31863",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23231374,
            "range": "± 39901",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42237162,
            "range": "± 38537",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 81741001,
            "range": "± 95692",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7380,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8694,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15870,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20291,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 29561,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 50043,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 103332,
            "range": "± 8602",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 200137,
            "range": "± 10687",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 408943,
            "range": "± 14538",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 858474,
            "range": "± 10908",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1803825,
            "range": "± 15403",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3812155,
            "range": "± 41219",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8139709,
            "range": "± 167840",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17600963,
            "range": "± 192207",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 37377602,
            "range": "± 203533",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 81144063,
            "range": "± 872478",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34855,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35005,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 178758835,
            "range": "± 833099",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 384109000,
            "range": "± 1897369",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 829941385,
            "range": "± 1481639",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1786689595,
            "range": "± 4998608",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3827297083,
            "range": "± 3289395",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8180966995,
            "range": "± 67767787",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17415591964,
            "range": "± 18291281",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36979403719,
            "range": "± 21901240",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 78218758763,
            "range": "± 64759300",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 110662588,
            "range": "± 613457",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 190267402,
            "range": "± 2538470",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 337302916,
            "range": "± 2194555",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 614451337,
            "range": "± 1254940",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1144615213,
            "range": "± 5414567",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2160992463,
            "range": "± 6052865",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4110972738,
            "range": "± 9893111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7936573650,
            "range": "± 15899786",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15259453664,
            "range": "± 24390515",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5989891,
            "range": "± 72116",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9267426,
            "range": "± 79632",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15142601,
            "range": "± 80193",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25411667,
            "range": "± 278189",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43309831,
            "range": "± 1471239",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 77163763,
            "range": "± 1511627",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137924338,
            "range": "± 776308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 251886282,
            "range": "± 3522871",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 460147011,
            "range": "± 2169261",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}