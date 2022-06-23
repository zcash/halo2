window.BENCHMARK_DATA = {
  "lastUpdate": 1656015368028,
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
          "id": "912003138f6711a289548086da8229c323ca7d7a",
          "message": "Merge pull request #611 from zcash/release-0.2.0\n\nRelease 0.2.0",
          "timestamp": "2022-06-23T12:35:40-06:00",
          "tree_id": "eccac1bbaae8f9311694906a6a974f80f110a1a3",
          "url": "https://github.com/zcash/halo2/commit/912003138f6711a289548086da8229c323ca7d7a"
        },
        "date": 1656013363799,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 86712750,
            "range": "± 6003586",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3513416,
            "range": "± 87354",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 204271814,
            "range": "± 2605044",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4204347,
            "range": "± 114634",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 274601062,
            "range": "± 2920390",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4584575,
            "range": "± 134778",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 44450,
            "range": "± 1413",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 161696,
            "range": "± 2298",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 175586,
            "range": "± 2293",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 284699,
            "range": "± 7311",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 283754,
            "range": "± 3781",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 168386,
            "range": "± 3119",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 183381,
            "range": "± 3006",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 290362,
            "range": "± 4829",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 289374,
            "range": "± 4066",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 349406,
            "range": "± 5466",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 367943,
            "range": "± 5763",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 479037,
            "range": "± 6825",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 471732,
            "range": "± 5932",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3507779,
            "range": "± 53795",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7005194,
            "range": "± 91956",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12054950,
            "range": "± 203793",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 25164650,
            "range": "± 522907",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 46924050,
            "range": "± 1460219",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 95004298,
            "range": "± 1502829",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8182,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 13296,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 17824,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21317,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 31043,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 53750,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 110848,
            "range": "± 9330",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 218872,
            "range": "± 10164",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 452292,
            "range": "± 21215",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 947932,
            "range": "± 38721",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2002295,
            "range": "± 75556",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4372106,
            "range": "± 78078",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9466435,
            "range": "± 215366",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 20355900,
            "range": "± 399972",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 46221676,
            "range": "± 2298674",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 104632470,
            "range": "± 5052973",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33013,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33010,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 180473536,
            "range": "± 2399993",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 382212807,
            "range": "± 2577525",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 829089288,
            "range": "± 8084967",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1801166791,
            "range": "± 21776890",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3845113703,
            "range": "± 83783516",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8087059283,
            "range": "± 45406815",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17546799118,
            "range": "± 214991477",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 36479401001,
            "range": "± 653990992",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 77504003805,
            "range": "± 1472599657",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 109063290,
            "range": "± 763486",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 188923423,
            "range": "± 1334615",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 334012771,
            "range": "± 3250422",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 612162502,
            "range": "± 6613982",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1151329385,
            "range": "± 5352300",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2153884173,
            "range": "± 3726549",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4092688456,
            "range": "± 71698097",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7968986717,
            "range": "± 97053603",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 15293539007,
            "range": "± 65019859",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5910950,
            "range": "± 145460",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9212134,
            "range": "± 327126",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15251822,
            "range": "± 196078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 25279669,
            "range": "± 1367446",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 43297758,
            "range": "± 1219783",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 76886318,
            "range": "± 1202818",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 139593158,
            "range": "± 2278958",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 261405911,
            "range": "± 5478448",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 469929785,
            "range": "± 8551602",
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
          "id": "96d9bde905a20117b4350ffba0b0a6479aa63f0a",
          "message": "Merge pull request #612 from zcash/batch-crate-feature\n\nPlace `halo2_proofs::plonk::BatchVerifier` behind a (default enabled) crate feature called `batch`.",
          "timestamp": "2022-06-23T20:16:37+01:00",
          "tree_id": "d7405628dd103edb941d0d31fd9d0c39a92f50d8",
          "url": "https://github.com/zcash/halo2/commit/96d9bde905a20117b4350ffba0b0a6479aa63f0a"
        },
        "date": 1656015362954,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74897656,
            "range": "± 1642453",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2957188,
            "range": "± 28509",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 173973521,
            "range": "± 1184036",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3620024,
            "range": "± 54487",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 239569059,
            "range": "± 1061987",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3988053,
            "range": "± 41434",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38793,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141530,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153851,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249797,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249572,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144567,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156833,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252568,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252647,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302500,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314758,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410544,
            "range": "± 477",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410647,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3100917,
            "range": "± 1514",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5978746,
            "range": "± 1698",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10340229,
            "range": "± 8988",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22384375,
            "range": "± 87932",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42868444,
            "range": "± 390190",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84462310,
            "range": "± 232660",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7564,
            "range": "± 2998",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8688,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16230,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19187,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27590,
            "range": "± 1083",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47159,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 96890,
            "range": "± 7620",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188164,
            "range": "± 8588",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 385616,
            "range": "± 11649",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 812681,
            "range": "± 23794",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1722003,
            "range": "± 56288",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3686735,
            "range": "± 123479",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8164848,
            "range": "± 54619",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17840157,
            "range": "± 177408",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38816364,
            "range": "± 455383",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 87217676,
            "range": "± 1091720",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28464,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28527,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153243060,
            "range": "± 301088",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327345848,
            "range": "± 685246",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 709198964,
            "range": "± 58042638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1521063789,
            "range": "± 2400903",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3264888418,
            "range": "± 11706361",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6974586102,
            "range": "± 15191301",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14856224974,
            "range": "± 21769513",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31535208006,
            "range": "± 53202323",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66672301342,
            "range": "± 44804980",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96351027,
            "range": "± 600377",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166750673,
            "range": "± 1435188",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 294609236,
            "range": "± 692600",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 538353998,
            "range": "± 2131440",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1008813854,
            "range": "± 2121078",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1902436049,
            "range": "± 7590001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3610896316,
            "range": "± 8368155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7000409216,
            "range": "± 17109121",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13469626560,
            "range": "± 72871352",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5134270,
            "range": "± 42022",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7990651,
            "range": "± 334985",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12858225,
            "range": "± 312281",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21746043,
            "range": "± 806809",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37521427,
            "range": "± 647199",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66998551,
            "range": "± 2190984",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119544570,
            "range": "± 845578",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222301870,
            "range": "± 4382600",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404997402,
            "range": "± 7599483",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}