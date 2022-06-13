window.BENCHMARK_DATA = {
  "lastUpdate": 1655138436279,
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
          "id": "a898d65ae3ad3d41987666f6a03cfc15edae01c4",
          "message": "Merge pull request #598 from zcash/circuit-value-type\n\nIntroduce `halo2_proofs::circuit::Value`",
          "timestamp": "2022-06-13T16:41:26+01:00",
          "tree_id": "3b5b3835114577cf10d58fbf72c97e83e7bc88d8",
          "url": "https://github.com/zcash/halo2/commit/a898d65ae3ad3d41987666f6a03cfc15edae01c4"
        },
        "date": 1655138429706,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74731865,
            "range": "± 4644522",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3366911,
            "range": "± 53292",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 175201253,
            "range": "± 2508720",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4930854,
            "range": "± 126960",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 240371253,
            "range": "± 1599654",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 6009194,
            "range": "± 66253",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38984,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141115,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153388,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 252467,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 252535,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144114,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156355,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 255386,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 255534,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 301502,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 313827,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412953,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 413125,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3069923,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5904773,
            "range": "± 1317",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10550820,
            "range": "± 27151",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22654700,
            "range": "± 83738",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 41480375,
            "range": "± 248116",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 82250654,
            "range": "± 319849",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7326,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8522,
            "range": "± 631",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15374,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 18601,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28080,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47634,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98152,
            "range": "± 7605",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188780,
            "range": "± 13545",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 387360,
            "range": "± 12624",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 815782,
            "range": "± 22449",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1721344,
            "range": "± 55934",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3683200,
            "range": "± 33006",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8147619,
            "range": "± 87210",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17735616,
            "range": "± 472108",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38689051,
            "range": "± 320658",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 89690213,
            "range": "± 747907",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28407,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28498,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 152667961,
            "range": "± 237082",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328023769,
            "range": "± 2529209",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 704242248,
            "range": "± 1054148",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1517745601,
            "range": "± 2315992",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3253161170,
            "range": "± 9570232",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6941013815,
            "range": "± 11916434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14796215137,
            "range": "± 23465183",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31440273418,
            "range": "± 44108635",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66560380218,
            "range": "± 57606826",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96310033,
            "range": "± 465339",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166509441,
            "range": "± 2985113",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 293718213,
            "range": "± 865852",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 536835012,
            "range": "± 1797643",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1006108796,
            "range": "± 2517598",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1897980798,
            "range": "± 4716579",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3615631042,
            "range": "± 10320508",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6964651950,
            "range": "± 13726902",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13450079613,
            "range": "± 28123735",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5174788,
            "range": "± 289277",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7998952,
            "range": "± 57279",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 13045546,
            "range": "± 105237",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21772081,
            "range": "± 161638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37800539,
            "range": "± 1161034",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67175858,
            "range": "± 529578",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120511095,
            "range": "± 3309704",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 220895737,
            "range": "± 9308536",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 401246230,
            "range": "± 3643981",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}