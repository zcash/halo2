window.BENCHMARK_DATA = {
  "lastUpdate": 1656008088081,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ewillbefull@gmail.com",
            "name": "ebfull",
            "username": "ebfull"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7087c33658ce3e2a341237f41ca14bfeb14da637",
          "message": "Merge pull request #608 from ebfull/msm-optimization\n\nMSM optimizations",
          "timestamp": "2022-06-23T10:49:22-06:00",
          "tree_id": "ac25f31afe974dd4e415a41c9d0e2ce12ae12562",
          "url": "https://github.com/zcash/halo2/commit/7087c33658ce3e2a341237f41ca14bfeb14da637"
        },
        "date": 1656006829389,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 79998810,
            "range": "± 5098934",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3477778,
            "range": "± 68267",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 180043604,
            "range": "± 3011409",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4248232,
            "range": "± 83780",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 253173087,
            "range": "± 1495775",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4659139,
            "range": "± 82735",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 48357,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 163968,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179221,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 292433,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 292398,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 167472,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 182667,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 295989,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 295937,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 350058,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 365401,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 478877,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 478648,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3612420,
            "range": "± 20849",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6123408,
            "range": "± 18342",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10617251,
            "range": "± 88728",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22422003,
            "range": "± 17317",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42238772,
            "range": "± 259267",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82533426,
            "range": "± 208584",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 6716,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 7911,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15367,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20164,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28665,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 50583,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 104014,
            "range": "± 6856",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 202260,
            "range": "± 10361",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 414738,
            "range": "± 11142",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 867573,
            "range": "± 11993",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1839743,
            "range": "± 50271",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3869418,
            "range": "± 35652",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8313691,
            "range": "± 72464",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17858693,
            "range": "± 376200",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38919686,
            "range": "± 226078",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 85588376,
            "range": "± 2582041",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34993,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35119,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 178271833,
            "range": "± 5472646",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 384556200,
            "range": "± 833994",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 830311114,
            "range": "± 3852416",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1785952355,
            "range": "± 7191009",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3824704162,
            "range": "± 2868362",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8180334485,
            "range": "± 36593849",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17424759827,
            "range": "± 27343963",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36951438159,
            "range": "± 52505867",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 78273975897,
            "range": "± 262420560",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 111023494,
            "range": "± 704629",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 191553944,
            "range": "± 1071540",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 338363552,
            "range": "± 35197424",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 614210193,
            "range": "± 2928055",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1143827720,
            "range": "± 2021720",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2135569051,
            "range": "± 12920987",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4096664514,
            "range": "± 5462256",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7906769942,
            "range": "± 15971378",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15198266418,
            "range": "± 30725477",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5918095,
            "range": "± 92274",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9165201,
            "range": "± 212515",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14944848,
            "range": "± 617277",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25156170,
            "range": "± 276168",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43210611,
            "range": "± 1145775",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 77242968,
            "range": "± 1523915",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 138350422,
            "range": "± 1241925",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 254540480,
            "range": "± 3655584",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 462248553,
            "range": "± 3516455",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "dac6cfb5d70c4672de672d57fa27b211eef9c997",
          "message": "Merge pull request #610 from zcash/rework-batch-verifier\n\nRework `halo2_proofs::plonk::BatchVerifier`",
          "timestamp": "2022-06-23T18:15:24+01:00",
          "tree_id": "91a2674938b6dbeba848efdb3d55d6e883eca60e",
          "url": "https://github.com/zcash/halo2/commit/dac6cfb5d70c4672de672d57fa27b211eef9c997"
        },
        "date": 1656008082776,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74312260,
            "range": "± 554213",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2950669,
            "range": "± 50450",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 173917305,
            "range": "± 4999773",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3649609,
            "range": "± 81281",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 237971679,
            "range": "± 4638045",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3986995,
            "range": "± 75638",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38711,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141678,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153820,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249808,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249871,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144741,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156921,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252780,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252720,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302816,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 315075,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410973,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410697,
            "range": "± 2873",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3004648,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5984408,
            "range": "± 17000",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10522325,
            "range": "± 40694",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23976634,
            "range": "± 121051",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43403501,
            "range": "± 116808",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 85150692,
            "range": "± 186410",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7618,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9565,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15815,
            "range": "± 547",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18900,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27723,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 48161,
            "range": "± 3382",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 97055,
            "range": "± 9409",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188582,
            "range": "± 8745",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 385404,
            "range": "± 15365",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 816373,
            "range": "± 27445",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1727920,
            "range": "± 54785",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3691088,
            "range": "± 61690",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8143503,
            "range": "± 116535",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17911547,
            "range": "± 467686",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 40624124,
            "range": "± 1352422",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 91044372,
            "range": "± 2475996",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28439,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28537,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153040898,
            "range": "± 5693710",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 329157621,
            "range": "± 3582204",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 708514678,
            "range": "± 2014781",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1520505242,
            "range": "± 6213263",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3254327549,
            "range": "± 4129095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6948209563,
            "range": "± 79445022",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14820393398,
            "range": "± 31558452",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31469397509,
            "range": "± 40392985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66696142268,
            "range": "± 86091603",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96894479,
            "range": "± 1103475",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 167098846,
            "range": "± 735093",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 297021028,
            "range": "± 7075574",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 541724072,
            "range": "± 2812718",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1013363910,
            "range": "± 3415685",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1913932119,
            "range": "± 9717457",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3628259694,
            "range": "± 175707594",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7010029306,
            "range": "± 19793070",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13460897024,
            "range": "± 59213895",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5142258,
            "range": "± 35236",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8000923,
            "range": "± 131510",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12787046,
            "range": "± 402073",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21844239,
            "range": "± 286980",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37786187,
            "range": "± 800754",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66384022,
            "range": "± 1019807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120450976,
            "range": "± 802503",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222131895,
            "range": "± 4032120",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 402128956,
            "range": "± 9418443",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}