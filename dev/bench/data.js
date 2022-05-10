window.BENCHMARK_DATA = {
  "lastUpdate": 1652189338162,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bdf84a5bb5f1d03492314a461d46e29eacbeb18d",
          "message": "Merge pull request #580 from zcash/book-patch-summary\n\n[book] Add 'Selector combining' to SUMMARY.md",
          "timestamp": "2022-05-10T05:21:29-07:00",
          "tree_id": "7ac28e44bb46cb8e2f0dc4885fbc9562a9707443",
          "url": "https://github.com/zcash/halo2/commit/bdf84a5bb5f1d03492314a461d46e29eacbeb18d"
        },
        "date": 1652189332697,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 85128071,
            "range": "± 6563632",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4099596,
            "range": "± 307833",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 193669270,
            "range": "± 8853744",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5984007,
            "range": "± 367970",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 270618564,
            "range": "± 12692998",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7168931,
            "range": "± 507367",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 42875,
            "range": "± 2470",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 154870,
            "range": "± 12174",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 177777,
            "range": "± 33938",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 289097,
            "range": "± 17230",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 293071,
            "range": "± 15071",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 163222,
            "range": "± 12275",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 180998,
            "range": "± 11155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 296870,
            "range": "± 20738",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 297670,
            "range": "± 23732",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 347997,
            "range": "± 27954",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 353780,
            "range": "± 20020",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 451712,
            "range": "± 29854",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 458295,
            "range": "± 23698",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3478197,
            "range": "± 192731",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6782556,
            "range": "± 235035",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 11355747,
            "range": "± 488463",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 25561507,
            "range": "± 1067757",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 46620495,
            "range": "± 2368227",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 89339717,
            "range": "± 5920501",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8827,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10136,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18581,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 24105,
            "range": "± 1437",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32975,
            "range": "± 2062",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 57403,
            "range": "± 5242",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 118297,
            "range": "± 10868",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 229393,
            "range": "± 31680",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 452401,
            "range": "± 31354",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 989998,
            "range": "± 64767",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2164467,
            "range": "± 162492",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4646949,
            "range": "± 280708",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9866993,
            "range": "± 744650",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21547640,
            "range": "± 1159205",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 46553188,
            "range": "± 2576770",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 101346374,
            "range": "± 5477101",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33876,
            "range": "± 2093",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 32456,
            "range": "± 2196",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 181079960,
            "range": "± 8344490",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 391766079,
            "range": "± 9264255",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 857233844,
            "range": "± 44313913",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1772362852,
            "range": "± 62431137",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3894373445,
            "range": "± 138127686",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8369861226,
            "range": "± 118163547",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17640985969,
            "range": "± 219624345",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37358345908,
            "range": "± 377570726",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 79455339342,
            "range": "± 894140610",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 112936638,
            "range": "± 4390729",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 199985984,
            "range": "± 8469622",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 349739752,
            "range": "± 16404544",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 625176140,
            "range": "± 24786591",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1174190646,
            "range": "± 31517184",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2221934323,
            "range": "± 63012330",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4280297848,
            "range": "± 125776872",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8294548336,
            "range": "± 88669492",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16045159619,
            "range": "± 145135925",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6023419,
            "range": "± 312762",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9580092,
            "range": "± 584447",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15208706,
            "range": "± 1053471",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 26516698,
            "range": "± 2399764",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43685222,
            "range": "± 2271914",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 77571365,
            "range": "± 3693160",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 142982068,
            "range": "± 7111701",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 263384694,
            "range": "± 10494457",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 477192270,
            "range": "± 21338552",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}