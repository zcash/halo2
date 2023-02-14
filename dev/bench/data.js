window.BENCHMARK_DATA = {
  "lastUpdate": 1676404607689,
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
          "id": "210483df8e144d07381f38f70fce170d29ed9ce2",
          "message": "Merge pull request #734 from zcash/inferno-msrv-1.59\n\nConstrain `inferno` transitive dependency to our MSRV",
          "timestamp": "2023-02-14T18:53:17Z",
          "tree_id": "94606d48de0560074a5cc027d7406b5489b141b6",
          "url": "https://github.com/zcash/halo2/commit/210483df8e144d07381f38f70fce170d29ed9ce2"
        },
        "date": 1676404597731,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 99851323,
            "range": "± 9462157",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 5455091,
            "range": "± 508655",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 184536414,
            "range": "± 4306161",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6793087,
            "range": "± 571932",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 251009730,
            "range": "± 5337621",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7640005,
            "range": "± 731593",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 40364,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 160293,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 175476,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 285103,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 285134,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 163671,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 178835,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 288397,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 288383,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 342325,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 357363,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 467018,
            "range": "± 182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 467089,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3466227,
            "range": "± 2027",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 5933246,
            "range": "± 8051",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 10250719,
            "range": "± 28945",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 21069491,
            "range": "± 189274",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 40317619,
            "range": "± 148572",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 79189648,
            "range": "± 182346",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 7101,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 8456,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 14933,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 19448,
            "range": "± 709",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 27061,
            "range": "± 3093",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 44799,
            "range": "± 7181",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 104305,
            "range": "± 13708",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 216046,
            "range": "± 37971",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 471659,
            "range": "± 57297",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 886706,
            "range": "± 109660",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 1677526,
            "range": "± 154249",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 3580704,
            "range": "± 400610",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 7904692,
            "range": "± 702269",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 16601233,
            "range": "± 2050843",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 35209453,
            "range": "± 3167840",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 74425390,
            "range": "± 1949319",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34575,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34687,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 173365184,
            "range": "± 569379",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 374622867,
            "range": "± 3853271",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 811351232,
            "range": "± 7419022",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1739254736,
            "range": "± 5615734",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 3705681502,
            "range": "± 12286072",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 7941215852,
            "range": "± 11353176",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 16907045741,
            "range": "± 24607757",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 35860260547,
            "range": "± 27054556",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 75827009534,
            "range": "± 58869576",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 118323512,
            "range": "± 4560313",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 205415274,
            "range": "± 6462135",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 349629506,
            "range": "± 8591855",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 614745429,
            "range": "± 10959764",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1142742043,
            "range": "± 14496892",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2122870981,
            "range": "± 16653156",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4050110846,
            "range": "± 14975879",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 7762791045,
            "range": "± 17650577",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 14873019542,
            "range": "± 53560862",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 5837169,
            "range": "± 347200",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9064270,
            "range": "± 676205",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 14494456,
            "range": "± 986832",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 24451785,
            "range": "± 2485121",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 42024747,
            "range": "± 1789173",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 75298054,
            "range": "± 5174400",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 135585496,
            "range": "± 6360911",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 248278011,
            "range": "± 10136442",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 452216445,
            "range": "± 6466544",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}