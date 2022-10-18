window.BENCHMARK_DATA = {
  "lastUpdate": 1666115418576,
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
          "id": "ec9dcefe9103fc23c13e8195120419d4d2f232a6",
          "message": "[book] Include WASM guide in SUMMARY.md",
          "timestamp": "2022-10-18T12:52:25-04:00",
          "tree_id": "d3dbaccafcee3be59142506b2e882f939d1180b7",
          "url": "https://github.com/zcash/halo2/commit/ec9dcefe9103fc23c13e8195120419d4d2f232a6"
        },
        "date": 1666115411783,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60646711,
            "range": "± 4796179",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2989278,
            "range": "± 35339",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 132293288,
            "range": "± 3883735",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3664681,
            "range": "± 225741",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 182347642,
            "range": "± 1464181",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3988919,
            "range": "± 34478",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38774,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 142303,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 154515,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249894,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249786,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 145338,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 157837,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252759,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252738,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 304217,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 316699,
            "range": "± 1776",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 411751,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 411538,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3346270,
            "range": "± 1546",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5959895,
            "range": "± 3821",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10359208,
            "range": "± 24783",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22751851,
            "range": "± 95008",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43260270,
            "range": "± 242436",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 83875971,
            "range": "± 166417",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7956,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8987,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15630,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18990,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28405,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47284,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98635,
            "range": "± 4685",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 190284,
            "range": "± 9190",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 387466,
            "range": "± 26095",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 818703,
            "range": "± 24121",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1724128,
            "range": "± 45478",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3710860,
            "range": "± 82555",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8157889,
            "range": "± 62355",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17807227,
            "range": "± 209219",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38324677,
            "range": "± 946750",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 88276612,
            "range": "± 1971414",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28486,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28558,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153685089,
            "range": "± 716984",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328043445,
            "range": "± 633859",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707713907,
            "range": "± 1346502",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1523806040,
            "range": "± 9536806",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3267779511,
            "range": "± 6493512",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6973307203,
            "range": "± 12027762",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14898438911,
            "range": "± 45333562",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31591955580,
            "range": "± 42029346",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66805886686,
            "range": "± 52013456",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95615464,
            "range": "± 1933031",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166359681,
            "range": "± 1928340",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 293174645,
            "range": "± 2631794",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 534340178,
            "range": "± 2381453",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 988813826,
            "range": "± 3182976",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1871512386,
            "range": "± 4889033",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3557614375,
            "range": "± 78572489",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6901888771,
            "range": "± 14402001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13287259606,
            "range": "± 209907450",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5190779,
            "range": "± 216388",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8048311,
            "range": "± 175103",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12842947,
            "range": "± 334128",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21795582,
            "range": "± 395944",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37551537,
            "range": "± 711835",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67156479,
            "range": "± 1009001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120403305,
            "range": "± 1808308",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222377421,
            "range": "± 9668349",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 405072627,
            "range": "± 1665178",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}