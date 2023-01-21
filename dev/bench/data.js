window.BENCHMARK_DATA = {
  "lastUpdate": 1674259921254,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "45801863+alexander-camuto@users.noreply.github.com",
            "name": "dante",
            "username": "alexander-camuto"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9eb8eadbd7c19bbdca5ec427f9870b077429a7dc",
          "message": "chore: instance columns for poseidon bench (#712)",
          "timestamp": "2023-01-20T23:08:24Z",
          "tree_id": "45ba55e3c9ec3038011e4b9365532ebd05f3f3cb",
          "url": "https://github.com/zcash/halo2/commit/9eb8eadbd7c19bbdca5ec427f9870b077429a7dc"
        },
        "date": 1674259911543,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 100924031,
            "range": "± 6199474",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5716856,
            "range": "± 610493",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 187031978,
            "range": "± 4870471",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6797858,
            "range": "± 701199",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 256069707,
            "range": "± 7112275",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7706635,
            "range": "± 742565",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 46866,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 164739,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179835,
            "range": "± 820",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 293234,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 293090,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168167,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183299,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 296490,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 296479,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 351817,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 367023,
            "range": "± 421",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 480042,
            "range": "± 2099",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 480046,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3774049,
            "range": "± 16579",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5962558,
            "range": "± 3689",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10213109,
            "range": "± 3303",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21766859,
            "range": "± 34940",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 39795589,
            "range": "± 348531",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 79194279,
            "range": "± 260095",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7272,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8561,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14794,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18943,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26792,
            "range": "± 2784",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43792,
            "range": "± 6320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 100979,
            "range": "± 14513",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 211502,
            "range": "± 33951",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 462199,
            "range": "± 56573",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 868544,
            "range": "± 113275",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1651365,
            "range": "± 170362",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3578443,
            "range": "± 381789",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7751107,
            "range": "± 660675",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15285874,
            "range": "± 1320705",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33240234,
            "range": "± 3298240",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 71789402,
            "range": "± 904836",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34889,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34997,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 177374488,
            "range": "± 659968",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 391729820,
            "range": "± 5651016",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 833186739,
            "range": "± 13725974",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1767969217,
            "range": "± 11735223",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3776197155,
            "range": "± 7004933",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8095934405,
            "range": "± 33397963",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17214753598,
            "range": "± 23601143",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36576014560,
            "range": "± 144068929",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77220695640,
            "range": "± 79527259",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 119066976,
            "range": "± 3691039",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 210104699,
            "range": "± 5835383",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 361853899,
            "range": "± 7400243",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 638791602,
            "range": "± 13830738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1147236227,
            "range": "± 11502539",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2160706589,
            "range": "± 15066920",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4099875002,
            "range": "± 30412636",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7890309687,
            "range": "± 17337805",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15116312151,
            "range": "± 27353547",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5969007,
            "range": "± 647843",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9259482,
            "range": "± 738389",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14841958,
            "range": "± 1019961",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24915203,
            "range": "± 2209797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43002280,
            "range": "± 3169602",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76585633,
            "range": "± 6081347",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 137718064,
            "range": "± 6796737",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 252797841,
            "range": "± 15376812",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 459662606,
            "range": "± 4500783",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}