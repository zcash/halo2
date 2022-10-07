window.BENCHMARK_DATA = {
  "lastUpdate": 1665155931348,
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
          "id": "c824785b7d61ec329de218d05cab20acfc671cea",
          "message": "Merge pull request #677 from zcash/therealyingtong-patch-1\n\n[doc] commitment::verifier: verify_proof does not have [-c]G term",
          "timestamp": "2022-10-07T15:11:07+01:00",
          "tree_id": "6fa6aac77e4eadee72dcf878b91638e8e1859633",
          "url": "https://github.com/zcash/halo2/commit/c824785b7d61ec329de218d05cab20acfc671cea"
        },
        "date": 1665155919456,
        "tool": "cargo",
        "benches": [
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 74680419,
            "range": "± 7401959",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 3597019,
            "range": "± 157303",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 162677411,
            "range": "± 2642135",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 4391461,
            "range": "± 298869",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 225923796,
            "range": "± 5947919",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 4886287,
            "range": "± 252009",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 48016,
            "range": "± 2182",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 171164,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 185819,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 301647,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 301694,
            "range": "± 876",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 174890,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 189582,
            "range": "± 2838",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 305272,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 305214,
            "range": "± 1757",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 365690,
            "range": "± 2281",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 380544,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 496016,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 496332,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "double-and-add",
            "value": 3732542,
            "range": "± 2294",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/14",
            "value": 7214390,
            "range": "± 90638",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/15",
            "value": 12530658,
            "range": "± 38384",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/16",
            "value": 27850827,
            "range": "± 174619",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/17",
            "value": 52320546,
            "range": "± 79065",
            "unit": "ns/iter"
          },
          {
            "name": "dev-lookup/18",
            "value": 99978976,
            "range": "± 549968",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/3",
            "value": 8947,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/4",
            "value": 10125,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/5",
            "value": 19104,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/6",
            "value": 22524,
            "range": "± 493",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/7",
            "value": 32721,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/8",
            "value": 56812,
            "range": "± 2366",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/9",
            "value": 120192,
            "range": "± 4705",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/10",
            "value": 233801,
            "range": "± 10792",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/11",
            "value": 473553,
            "range": "± 22027",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/12",
            "value": 1005058,
            "range": "± 52230",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/13",
            "value": 2171807,
            "range": "± 74067",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/14",
            "value": 4663566,
            "range": "± 161934",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/15",
            "value": 10066531,
            "range": "± 344091",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/16",
            "value": 22333816,
            "range": "± 706700",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/17",
            "value": 48344001,
            "range": "± 1528990",
            "unit": "ns/iter"
          },
          {
            "name": "fft/k/18",
            "value": 107417008,
            "range": "± 4561287",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Pallas",
            "value": 34186,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "hash-to-curve/Vesta",
            "value": 34395,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/8",
            "value": 193299245,
            "range": "± 12878021",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/9",
            "value": 409601056,
            "range": "± 11604569",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/10",
            "value": 889373553,
            "range": "± 10181408",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/11",
            "value": 1899092576,
            "range": "± 27967576",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/12",
            "value": 4081054558,
            "range": "± 61637638",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/13",
            "value": 8477247074,
            "range": "± 73994001",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/14",
            "value": 17896597840,
            "range": "± 196902232",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/15",
            "value": 37997154275,
            "range": "± 236964958",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-keygen/16",
            "value": 80213090683,
            "range": "± 404329551",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/8",
            "value": 115312120,
            "range": "± 1348605",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/9",
            "value": 200954085,
            "range": "± 5268700",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/10",
            "value": 358778668,
            "range": "± 9808433",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/11",
            "value": 665828647,
            "range": "± 15635138",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/12",
            "value": 1242145151,
            "range": "± 12055231",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/13",
            "value": 2269576944,
            "range": "± 36913978",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/14",
            "value": 4340440498,
            "range": "± 65682258",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/15",
            "value": 8377680137,
            "range": "± 60348654",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-prover/16",
            "value": 16065429743,
            "range": "± 193347429",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/8",
            "value": 6200336,
            "range": "± 115882",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/9",
            "value": 9878477,
            "range": "± 483569",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/10",
            "value": 15594026,
            "range": "± 901518",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/11",
            "value": 27222431,
            "range": "± 1574177",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/12",
            "value": 47092518,
            "range": "± 1616434",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/13",
            "value": 80995125,
            "range": "± 3157747",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/14",
            "value": 143943442,
            "range": "± 5041738",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/15",
            "value": 267581793,
            "range": "± 8271212",
            "unit": "ns/iter"
          },
          {
            "name": "plonk-verifier/16",
            "value": 485937158,
            "range": "± 11331435",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}