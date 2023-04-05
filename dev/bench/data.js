window.BENCHMARK_DATA = {
  "lastUpdate": 1680654321760,
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
          "id": "263356784042d7d4c1c17d357c94c1acaeb75ab5",
          "message": "Merge pull request #758 from zcash/release-0.3.0\n\nRelease 0.3.0 of halo2_proofs and halo2_gadgets",
          "timestamp": "2023-03-22T12:32:26-06:00",
          "tree_id": "16087d2e72223f8ad75a504c7125d88b237fc70a",
          "url": "https://github.com/zcash/halo2/commit/263356784042d7d4c1c17d357c94c1acaeb75ab5"
        },
        "date": 1679514083317,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 105831661,
            "range": "± 10291008",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 6147564,
            "range": "± 633168",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 198382172,
            "range": "± 9248835",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6783707,
            "range": "± 945761",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 273633600,
            "range": "± 11982269",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 8129904,
            "range": "± 791767",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 50798,
            "range": "± 2155",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 167764,
            "range": "± 8616",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 176975,
            "range": "± 7220",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 293677,
            "range": "± 17030",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 297425,
            "range": "± 14820",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168454,
            "range": "± 7169",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 185979,
            "range": "± 9886",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 304185,
            "range": "± 17118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 300779,
            "range": "± 15396",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 350940,
            "range": "± 19115",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 363313,
            "range": "± 21555",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 483046,
            "range": "± 24266",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 485819,
            "range": "± 23653",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3817301,
            "range": "± 175560",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7140577,
            "range": "± 200877",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 11963110,
            "range": "± 654626",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 24887630,
            "range": "± 477093",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 46686914,
            "range": "± 952372",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 93438734,
            "range": "± 2330970",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7201,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 9084,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16393,
            "range": "± 1314",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 20062,
            "range": "± 1703",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 30097,
            "range": "± 4614",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 50869,
            "range": "± 8320",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 122400,
            "range": "± 20616",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 262139,
            "range": "± 47610",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 562008,
            "range": "± 82945",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 994509,
            "range": "± 130074",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1974794,
            "range": "± 135757",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4312120,
            "range": "± 528256",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9040044,
            "range": "± 914044",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 19515351,
            "range": "± 2077692",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 44687878,
            "range": "± 3124310",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 90388906,
            "range": "± 3988445",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33070,
            "range": "± 1180",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 35926,
            "range": "± 1976",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 194196631,
            "range": "± 5169036",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 431388839,
            "range": "± 10644832",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 875619656,
            "range": "± 15349608",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1881512772,
            "range": "± 31187156",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3990531093,
            "range": "± 83913665",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8679901762,
            "range": "± 201762236",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18258801273,
            "range": "± 192603320",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 38889238182,
            "range": "± 703475862",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 81824193660,
            "range": "± 621092484",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 127546138,
            "range": "± 5254664",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 224012037,
            "range": "± 7720797",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 382582458,
            "range": "± 14724985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 673583752,
            "range": "± 16992938",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1235117985,
            "range": "± 21999029",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2323746274,
            "range": "± 26424557",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4374212079,
            "range": "± 57777648",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8483402727,
            "range": "± 120845880",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16244209460,
            "range": "± 136067912",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6375146,
            "range": "± 607163",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9981694,
            "range": "± 1192873",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16765485,
            "range": "± 2260280",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 28260622,
            "range": "± 2558140",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 46712806,
            "range": "± 3029684",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 84028949,
            "range": "± 6535196",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 154468264,
            "range": "± 8622535",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 275273617,
            "range": "± 17296965",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 504415123,
            "range": "± 14130782",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "5678a506cbec60d593a21ff618d09bed48abf7f2",
          "message": "Merge pull request #755 from zcash/lookup-errors\n\n`plonk::Error::TableError`: Better lookup errors",
          "timestamp": "2023-04-04T23:27:28Z",
          "tree_id": "e656b9132260d133ef8fc73654d012ae00242ca4",
          "url": "https://github.com/zcash/halo2/commit/5678a506cbec60d593a21ff618d09bed48abf7f2"
        },
        "date": 1680654314811,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 91834741,
            "range": "± 6066678",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4556154,
            "range": "± 442976",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 168865822,
            "range": "± 5316592",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 5542546,
            "range": "± 585775",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 227910597,
            "range": "± 6492549",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6520793,
            "range": "± 599524",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 39616,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141425,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153778,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 248739,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249064,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144355,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156803,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 251476,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 251860,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 301983,
            "range": "± 2825",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314850,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 409228,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 409193,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 2997154,
            "range": "± 1674",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5956996,
            "range": "± 22790",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10151542,
            "range": "± 57327",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21991814,
            "range": "± 517465",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42634771,
            "range": "± 717786",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82894015,
            "range": "± 596636",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7587,
            "range": "± 805",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8739,
            "range": "± 1646",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14928,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18252,
            "range": "± 1196",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 26198,
            "range": "± 2084",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 43871,
            "range": "± 6801",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98301,
            "range": "± 16712",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 208709,
            "range": "± 36731",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 447882,
            "range": "± 59954",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 846234,
            "range": "± 102853",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1627426,
            "range": "± 163347",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3258796,
            "range": "± 343289",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 6974177,
            "range": "± 313874",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 15436022,
            "range": "± 978044",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 33599973,
            "range": "± 1232877",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 79109636,
            "range": "± 1514686",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28499,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28633,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153232559,
            "range": "± 4659362",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 325746744,
            "range": "± 73950174",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 722219431,
            "range": "± 12817145",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1504673799,
            "range": "± 12093722",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3226260255,
            "range": "± 11245334",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6896764938,
            "range": "± 15338901",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14698647788,
            "range": "± 66875280",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31223785211,
            "range": "± 26071095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 65970131068,
            "range": "± 74405156",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 101234665,
            "range": "± 2792031",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 183209116,
            "range": "± 4693622",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 314150239,
            "range": "± 6324629",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 553862482,
            "range": "± 9496410",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1015135321,
            "range": "± 13580966",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1887790620,
            "range": "± 21721681",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3612044509,
            "range": "± 14771276",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6948648368,
            "range": "± 28787895",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13284707030,
            "range": "± 126711804",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5058425,
            "range": "± 389137",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7918904,
            "range": "± 753839",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12743180,
            "range": "± 899835",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21629074,
            "range": "± 1843557",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37176364,
            "range": "± 2498328",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66643740,
            "range": "± 4412465",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119606399,
            "range": "± 5696245",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 220965410,
            "range": "± 11812004",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 405820063,
            "range": "± 7638704",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}