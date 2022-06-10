window.BENCHMARK_DATA = {
  "lastUpdate": 1654822654210,
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
          "id": "1cf376e9a60448571df6049ffeb3f4c801b01260",
          "message": "Merge pull request #597 from zcash/small-perf-improvements\n\nSmall performance improvements",
          "timestamp": "2022-06-10T00:48:23+01:00",
          "tree_id": "08097aa58a06d02432ec3de854210b2e830fb5e5",
          "url": "https://github.com/zcash/halo2/commit/1cf376e9a60448571df6049ffeb3f4c801b01260"
        },
        "date": 1654822646466,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 90852315,
            "range": "± 3811584",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4043210,
            "range": "± 181476",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 211195844,
            "range": "± 7689554",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6011630,
            "range": "± 383159",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 288451856,
            "range": "± 7214452",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7244816,
            "range": "± 344339",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 45346,
            "range": "± 2301",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 169900,
            "range": "± 6448",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 181649,
            "range": "± 8571",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 299344,
            "range": "± 13148",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 290745,
            "range": "± 12525",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 171390,
            "range": "± 6791",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 187796,
            "range": "± 12184",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 301065,
            "range": "± 14892",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 293355,
            "range": "± 15433",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 354549,
            "range": "± 15448",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 368059,
            "range": "± 17678",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 476020,
            "range": "± 17542",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 479187,
            "range": "± 19325",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3647471,
            "range": "± 179917",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 6892138,
            "range": "± 76935",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12110343,
            "range": "± 218314",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 26326701,
            "range": "± 709193",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 48503393,
            "range": "± 1701123",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 93547082,
            "range": "± 1444172",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8880,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10183,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 18198,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 21799,
            "range": "± 3262",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 33657,
            "range": "± 2296",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 55501,
            "range": "± 4042",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 118294,
            "range": "± 10011",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 227317,
            "range": "± 16888",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 467441,
            "range": "± 31818",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 985737,
            "range": "± 63322",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2132660,
            "range": "± 123191",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4581739,
            "range": "± 170932",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 9953552,
            "range": "± 448959",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 21949081,
            "range": "± 1007361",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 47283432,
            "range": "± 1821233",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 106509259,
            "range": "± 4259574",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33249,
            "range": "± 1478",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33341,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 184079319,
            "range": "± 2840385",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 398108820,
            "range": "± 5362593",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 850149767,
            "range": "± 23027336",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1822973681,
            "range": "± 20431512",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3891915034,
            "range": "± 63375095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8345591825,
            "range": "± 65562032",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17840982929,
            "range": "± 106889113",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37778027088,
            "range": "± 172794435",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 80032311523,
            "range": "± 284593643",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 118311016,
            "range": "± 2442443",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 199867064,
            "range": "± 3466415",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 366978239,
            "range": "± 7962410",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 657995211,
            "range": "± 9653105",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1248536609,
            "range": "± 20113625",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2301614331,
            "range": "± 33593162",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4362230394,
            "range": "± 42384196",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8526408215,
            "range": "± 56815512",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16153117736,
            "range": "± 88324447",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6261559,
            "range": "± 289630",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9908692,
            "range": "± 790253",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15646227,
            "range": "± 880885",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27259421,
            "range": "± 1570719",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 45305869,
            "range": "± 1638733",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80708374,
            "range": "± 3064679",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 144940621,
            "range": "± 5472155",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 266391990,
            "range": "± 6087371",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 488212659,
            "range": "± 11747941",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}