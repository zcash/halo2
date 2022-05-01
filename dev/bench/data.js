window.BENCHMARK_DATA = {
  "lastUpdate": 1651422573268,
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
          "id": "1ccef3b30ab4b585e0bca39e96d7eb5b0d5c413e",
          "message": "Merge pull request #566 from zcash/ci-2-fix-2-bench\n\nhalo2: Disable default benchmark harness",
          "timestamp": "2022-05-01T16:25:44+01:00",
          "tree_id": "19f8528f4e2d7505b0359b0a3c773e34aa05ce9a",
          "url": "https://github.com/zcash/halo2/commit/1ccef3b30ab4b585e0bca39e96d7eb5b0d5c413e"
        },
        "date": 1651422569877,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 61207225,
            "range": "± 8341445",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3414170,
            "range": "± 115084",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 133493670,
            "range": "± 28076567",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5064811,
            "range": "± 71826",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 187676208,
            "range": "± 2036397",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6120225,
            "range": "± 70880",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 36286,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141839,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153969,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 254429,
            "range": "± 1954",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 254307,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144783,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156982,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 257297,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 257333,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302768,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 315030,
            "range": "± 1084",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 415503,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 415488,
            "range": "± 1394",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2960074,
            "range": "± 1911",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5862271,
            "range": "± 17824",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10402119,
            "range": "± 124331",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 23461113,
            "range": "± 246809",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 43271931,
            "range": "± 230722",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 85201734,
            "range": "± 344994",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 14849,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 16943,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 20886,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 28791,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 46227,
            "range": "± 1389",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 82057,
            "range": "± 1830",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 156548,
            "range": "± 9293",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 316009,
            "range": "± 7322",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 662689,
            "range": "± 20961",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1427084,
            "range": "± 58257",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 3026728,
            "range": "± 146394",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 6325283,
            "range": "± 113228",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 13434109,
            "range": "± 288007",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 28808576,
            "range": "± 617093",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 61862914,
            "range": "± 1711752",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 129553367,
            "range": "± 2199133",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28526,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28593,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 194336891,
            "range": "± 11222988",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 410742036,
            "range": "± 1106027",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 877471669,
            "range": "± 4290721",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1853580739,
            "range": "± 9693747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3918061072,
            "range": "± 4445403",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8306827842,
            "range": "± 28674096",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17504731014,
            "range": "± 23660165",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36764325929,
            "range": "± 75891145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77276282463,
            "range": "± 527276349",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 97694414,
            "range": "± 1527607",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 169678037,
            "range": "± 2973618",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 301939993,
            "range": "± 1368642",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 548167893,
            "range": "± 1904566",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1027921295,
            "range": "± 5932636",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1945295582,
            "range": "± 5244601",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3706114211,
            "range": "± 15025070",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7163531322,
            "range": "± 25421807",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13811966527,
            "range": "± 45689599",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5224807,
            "range": "± 57773",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8031529,
            "range": "± 47605",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13153157,
            "range": "± 662830",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 22192325,
            "range": "± 173962",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 38041865,
            "range": "± 434454",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67671976,
            "range": "± 3207593",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120866916,
            "range": "± 591078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 223166157,
            "range": "± 1959536",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404581463,
            "range": "± 4042364",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}