window.BENCHMARK_DATA = {
  "lastUpdate": 1657899154345,
  "repoUrl": "https://github.com/zcash/halo2",
  "entries": {
    "halo2 Benchmark": [
      {
        "commit": {
          "author": {
            "email": "tinghan0110@gmail.com",
            "name": "han0110",
            "username": "han0110"
          },
          "committer": {
            "email": "yingtong@z.cash",
            "name": "ying tong",
            "username": "therealyingtong"
          },
          "distinct": true,
          "id": "8ff5b1e3af95ce8e9dcf1e74173d3d62ad1770b0",
          "message": "feat: make `Expression::{Fixed,Advice,Instance}` to wrap their own `Query` struct",
          "timestamp": "2022-07-15T10:33:47-04:00",
          "tree_id": "4e392a8c081fd14d6c62fad128b58f041be30bd1",
          "url": "https://github.com/zcash/halo2/commit/8ff5b1e3af95ce8e9dcf1e74173d3d62ad1770b0"
        },
        "date": 1657899148733,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 61014173,
            "range": "± 548142",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2982043,
            "range": "± 448540",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 133138074,
            "range": "± 3680228",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3631063,
            "range": "± 64119",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 186297371,
            "range": "± 4226372",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3954595,
            "range": "± 53407",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38639,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 142717,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 154250,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 251387,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 251320,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 145738,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 157277,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 254367,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 254396,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 304924,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 315736,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 412268,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 412249,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3049057,
            "range": "± 17013",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5896731,
            "range": "± 3110",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10156088,
            "range": "± 21454",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22756235,
            "range": "± 208041",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42829026,
            "range": "± 309142",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84256901,
            "range": "± 143691",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7370,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8725,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16098,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19052,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27556,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47325,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 96719,
            "range": "± 7478",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188011,
            "range": "± 13264",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 386480,
            "range": "± 12335",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 813287,
            "range": "± 47823",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1718383,
            "range": "± 50369",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3694179,
            "range": "± 59215",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8134863,
            "range": "± 110455",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17861876,
            "range": "± 177377",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 38829073,
            "range": "± 421932",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 88645381,
            "range": "± 1959016",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28509,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28604,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153236109,
            "range": "± 4001535",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 327963026,
            "range": "± 628221",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 706722838,
            "range": "± 1812610",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1520966361,
            "range": "± 1664344",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3256061653,
            "range": "± 8463732",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6966693596,
            "range": "± 7321820",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14850886199,
            "range": "± 19888218",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31509608607,
            "range": "± 22140284",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66683819715,
            "range": "± 228552937",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 96309420,
            "range": "± 486668",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166340588,
            "range": "± 725437",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 294605091,
            "range": "± 934108",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 537900035,
            "range": "± 2959774",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1007004022,
            "range": "± 3672730",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1901326008,
            "range": "± 10819360",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3614961090,
            "range": "± 12245791",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6991122286,
            "range": "± 9536485",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13465925875,
            "range": "± 30021836",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5134095,
            "range": "± 51314",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7994527,
            "range": "± 320174",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12899013,
            "range": "± 254715",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21856101,
            "range": "± 754884",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37540921,
            "range": "± 857514",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67128463,
            "range": "± 1820028",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120179511,
            "range": "± 1636861",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222460261,
            "range": "± 3801696",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 404254877,
            "range": "± 4013660",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}