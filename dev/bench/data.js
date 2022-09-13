window.BENCHMARK_DATA = {
  "lastUpdate": 1663081877190,
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
          "id": "d5200aea5805dfade6c26d6e4e6c4e5072313502",
          "message": "Merge pull request #653 from zcash/minor-fixes\n\nMinor fixes",
          "timestamp": "2022-09-13T08:01:58-06:00",
          "tree_id": "6eaaea1a70441b710a8566b81d2d4086f5b8c65e",
          "url": "https://github.com/zcash/halo2/commit/d5200aea5805dfade6c26d6e4e6c4e5072313502"
        },
        "date": 1663081163288,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60944211,
            "range": "± 4507255",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2974710,
            "range": "± 52930",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 133070707,
            "range": "± 3343045",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3624915,
            "range": "± 62501",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 184971782,
            "range": "± 2063981",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4073268,
            "range": "± 67560",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38944,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 143240,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 155458,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 249817,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 249750,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 146187,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 158560,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 252707,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 252607,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 305990,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 318399,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 410618,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 410658,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3052143,
            "range": "± 2859",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5908844,
            "range": "± 11948",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10300789,
            "range": "± 46901",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21515884,
            "range": "± 141350",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42647102,
            "range": "± 179960",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84595075,
            "range": "± 148788",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7597,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 11790,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 15952,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19555,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27717,
            "range": "± 978",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 46699,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98249,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 188747,
            "range": "± 10066",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 386532,
            "range": "± 13140",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 818424,
            "range": "± 23067",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1727260,
            "range": "± 137941",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3704916,
            "range": "± 131273",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8200702,
            "range": "± 177453",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 17902248,
            "range": "± 1778904",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 39355759,
            "range": "± 445114",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 89171664,
            "range": "± 2254129",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28413,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28538,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153782444,
            "range": "± 3532366",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 328496030,
            "range": "± 858411",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707620120,
            "range": "± 1869220",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1523603837,
            "range": "± 9174499",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3262134393,
            "range": "± 4381201",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6978227110,
            "range": "± 9442447",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14887018280,
            "range": "± 51940152",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31587632024,
            "range": "± 53958443",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66933398913,
            "range": "± 107638102",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95683762,
            "range": "± 756351",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166829011,
            "range": "± 1627444",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 293690362,
            "range": "± 1093793",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 535522845,
            "range": "± 2864677",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 990544490,
            "range": "± 4162940",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1880117684,
            "range": "± 11322191",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3562421252,
            "range": "± 8453897",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6881623073,
            "range": "± 16497055",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13310500287,
            "range": "± 21757676",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5154888,
            "range": "± 118123",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 7962946,
            "range": "± 117886",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12885581,
            "range": "± 162878",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21887882,
            "range": "± 516988",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37556598,
            "range": "± 1100563",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 66726207,
            "range": "± 1989738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 119272092,
            "range": "± 1416951",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 221816733,
            "range": "± 4563715",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 403809213,
            "range": "± 2922575",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "61a1f63fcbc4792aaf1c285fd79a001495bd32f2",
          "message": "Merge pull request #623 from zcash/table-col-equality\n\nAllow `enable_equality` on `TableColumn`",
          "timestamp": "2022-09-13T08:03:20-06:00",
          "tree_id": "361cdadf3d724c37334bdc98e403d10203cbee30",
          "url": "https://github.com/zcash/halo2/commit/61a1f63fcbc4792aaf1c285fd79a001495bd32f2"
        },
        "date": 1663081237795,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 60756938,
            "range": "± 4825970",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 2973831,
            "range": "± 102974",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 132269291,
            "range": "± 2922152",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 3721393,
            "range": "± 60287",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 182811976,
            "range": "± 2401599",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 3953265,
            "range": "± 62663",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 38725,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 141637,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 153782,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 248895,
            "range": "± 2704",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 248776,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 144537,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 156821,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 251731,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 251793,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 302510,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 314696,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 409589,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 409528,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3038835,
            "range": "± 4510",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5904961,
            "range": "± 10868",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10857132,
            "range": "± 44621",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 22195739,
            "range": "± 99464",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 42602754,
            "range": "± 111335",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 84326404,
            "range": "± 199795",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7799,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8928,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 16457,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19846,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 28046,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 47698,
            "range": "± 3497",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 98494,
            "range": "± 3318",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 189322,
            "range": "± 10407",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 388635,
            "range": "± 10928",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 819194,
            "range": "± 22376",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1730006,
            "range": "± 95134",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3708525,
            "range": "± 76261",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 8182080,
            "range": "± 121407",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 18021838,
            "range": "± 146224",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 39165899,
            "range": "± 464043",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 88825353,
            "range": "± 1107335",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 28419,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 28550,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 153219028,
            "range": "± 315232",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 329084137,
            "range": "± 4418891",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 707087252,
            "range": "± 1449485",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1520689505,
            "range": "± 4473752",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3259695591,
            "range": "± 5479972",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 6974995468,
            "range": "± 181907219",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 14839958157,
            "range": "± 23363146",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 31533626186,
            "range": "± 33208517",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 66666693342,
            "range": "± 144238974",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 95891282,
            "range": "± 694111",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 166102350,
            "range": "± 906300",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 292059259,
            "range": "± 632924",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 532513270,
            "range": "± 4749755",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 989207990,
            "range": "± 2943483",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 1875715626,
            "range": "± 4689252",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 3562392025,
            "range": "± 11961213",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 6870018576,
            "range": "± 17707360",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 13254576740,
            "range": "± 31452331",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5200458,
            "range": "± 94304",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 8054790,
            "range": "± 199498",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 12845219,
            "range": "± 422370",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 21827416,
            "range": "± 650282",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 37651289,
            "range": "± 1161095",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 67188016,
            "range": "± 1589038",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 120516176,
            "range": "± 1288642",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 222510292,
            "range": "± 5099151",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 405280735,
            "range": "± 3791703",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "2bdb369393c11dfd093d68f9253e8f12e9e6281a",
          "message": "Merge pull request #654 from zcash/bug-instances-len\n\n`plonk::prover::create_proof`: Check that `circuits` and `instances` have equal lengths",
          "timestamp": "2022-09-13T08:02:26-06:00",
          "tree_id": "141dbf2eda804f47262c24ed35c7fc9b9eaa0b11",
          "url": "https://github.com/zcash/halo2/commit/2bdb369393c11dfd093d68f9253e8f12e9e6281a"
        },
        "date": 1663081867420,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 76746935,
            "range": "± 4885658",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3685086,
            "range": "± 288174",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 158792327,
            "range": "± 13597757",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4711003,
            "range": "± 282577",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 219707074,
            "range": "± 8432898",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 5033229,
            "range": "± 352792",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 50713,
            "range": "± 3181",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 176217,
            "range": "± 10609",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 179414,
            "range": "± 11832",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 306747,
            "range": "± 22284",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 312109,
            "range": "± 14851",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 178104,
            "range": "± 10427",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 197469,
            "range": "± 31566",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 302099,
            "range": "± 15875",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 320945,
            "range": "± 20278",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 371782,
            "range": "± 22277",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 381169,
            "range": "± 39587",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 514733,
            "range": "± 31813",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 476462,
            "range": "± 43433",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3896303,
            "range": "± 252124",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7741519,
            "range": "± 447066",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12700056,
            "range": "± 376250",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 28687163,
            "range": "± 1477787",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 55287766,
            "range": "± 2148338",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99743929,
            "range": "± 3485325",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8772,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 11119,
            "range": "± 1724",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 19178,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 23401,
            "range": "± 1602",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 34502,
            "range": "± 2334",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 58705,
            "range": "± 5993",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 126754,
            "range": "± 11340",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 240259,
            "range": "± 43793",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 500186,
            "range": "± 55285",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1054233,
            "range": "± 112621",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2248531,
            "range": "± 158226",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4797289,
            "range": "± 341721",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10506145,
            "range": "± 514153",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22279183,
            "range": "± 1079501",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 49534087,
            "range": "± 2471725",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 108154358,
            "range": "± 4412531",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 33853,
            "range": "± 1937",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 33531,
            "range": "± 1661",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 188090523,
            "range": "± 5297072",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 429372135,
            "range": "± 8633502",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 889436186,
            "range": "± 21826247",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1917176011,
            "range": "± 36145698",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4047173258,
            "range": "± 113150235",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8729787620,
            "range": "± 177931217",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 18607260562,
            "range": "± 506523535",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 39557394513,
            "range": "± 864454552",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 83474993239,
            "range": "± 1239747735",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 120333749,
            "range": "± 4871641",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 208610395,
            "range": "± 7002563",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 361762795,
            "range": "± 10589915",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 669078655,
            "range": "± 17787402",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1237475194,
            "range": "± 22353172",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2379948917,
            "range": "± 41434290",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4414600136,
            "range": "± 86234218",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8652170793,
            "range": "± 101063449",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16698695024,
            "range": "± 284926139",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6473156,
            "range": "± 484021",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 10248649,
            "range": "± 617324",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 16197223,
            "range": "± 1136972",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27783617,
            "range": "± 2224406",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 47714108,
            "range": "± 2543649",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80301732,
            "range": "± 4249359",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 152111519,
            "range": "± 5846074",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 282525180,
            "range": "± 12063149",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 501193052,
            "range": "± 16194330",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}