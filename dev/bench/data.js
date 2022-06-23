window.BENCHMARK_DATA = {
  "lastUpdate": 1656006834767,
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
      }
    ]
  }
}