package keyproof

import (
	"github.com/BeardOfDoom/pq-gabi/big"
	"github.com/BeardOfDoom/pq-gabi/internal/common"
	"github.com/BeardOfDoom/pq-gabi/zkproof"
)

type (
	rangeProofStructure struct {
		zkproof.RepresentationProofStructure
		rangeSecret string
		l1          uint
		l2          uint
	}

	RangeProof struct {
		Results map[string][]*big.Int
	}

	rangeCommit struct {
		commits map[string][]*big.Int
	}

	rangeCommitSecretLookup struct {
		rangeCommit
		i int
	}
)

func (s *rangeProofStructure) commitmentsFromSecrets(g zkproof.Group, list []*big.Int, bases zkproof.BaseLookup, secretdata zkproof.SecretLookup) ([]*big.Int, rangeCommit) {
	var commit rangeCommitSecretLookup

	// Build up commit data structure
	commit.commits = map[string][]*big.Int{}
	for _, curRhs := range s.Rhs {
		commit.commits[curRhs.Secret] = []*big.Int{}
	}

	// Some constants for commitment generation
	genLimit := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon+1)
	genOffset := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon)

	// Build up the range proof randomizers
	for i := 0; i < rangeProofIters; i++ {
		for name, clist := range commit.commits {
			var rval *big.Int
			if name == s.rangeSecret {
				rval = common.FastRandomBigInt(genLimit)
				rval.Sub(rval, genOffset)
			} else {
				rval = common.FastRandomBigInt(g.Order)
			}
			commit.commits[name] = append(clist, rval)
		}
	}

	// Construct the commitments
	secretMerge := zkproof.NewSecretMerge(&commit, secretdata)
	for i := 0; i < rangeProofIters; i++ {
		commit.i = i
		list = s.RepresentationProofStructure.CommitmentsFromSecrets(g, list, bases, &secretMerge)
	}

	// Call the logger
	Follower.Tick()

	// Return the result
	return list, commit.rangeCommit
}

func (s *rangeProofStructure) buildProof(g zkproof.Group, challenge *big.Int, commit rangeCommit, secretdata zkproof.SecretLookup) RangeProof {
	// For every value, build up results, handling the secret data separately
	proof := RangeProof{map[string][]*big.Int{}}
	for name, clist := range commit.commits {

		var rlist []*big.Int
		if name == s.rangeSecret {
			// special treatment for range secret
			resultOffset := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon+1)
			l1Offset := new(big.Int).Lsh(big.NewInt(1), s.l1)
			for i := 0; i < rangeProofIters; i++ {
				var res *big.Int
				if challenge.Bit(i) == 1 {
					res = new(big.Int).Sub(new(big.Int).Add(clist[i], l1Offset), secretdata.Secret(name))
				} else {
					res = new(big.Int).Set(clist[i])
				}
				res.Add(res, resultOffset)
				rlist = append(rlist, res)
			}
		} else {
			for i := 0; i < rangeProofIters; i++ {
				var res *big.Int
				if challenge.Bit(i) == 1 {
					res = new(big.Int).Mod(new(big.Int).Sub(clist[i], secretdata.Secret(name)), g.Order)
				} else {
					res = new(big.Int).Set(clist[i])
				}
				rlist = append(rlist, res)
			}
		}
		proof.Results[name] = rlist
	}

	return proof
}

func (s *rangeProofStructure) fakeProof(g zkproof.Group) RangeProof {
	// Some setup
	genLimit := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon+1)

	proof := RangeProof{map[string][]*big.Int{}}
	for _, curRhs := range s.Rhs {
		if curRhs.Secret == s.rangeSecret {
			var rlist []*big.Int
			for i := 0; i < rangeProofIters; i++ {
				rlist = append(rlist, common.FastRandomBigInt(genLimit))
			}
			proof.Results[curRhs.Secret] = rlist
		} else {
			var rlist []*big.Int
			for i := 0; i < rangeProofIters; i++ {
				rlist = append(rlist, common.FastRandomBigInt(g.Order))
			}
			proof.Results[curRhs.Secret] = rlist
		}
	}

	return proof
}

func (s *rangeProofStructure) verifyProofStructure(proof RangeProof) bool {
	// Validate presence of map
	if proof.Results == nil {
		return false
	}

	// Validate presence of all values
	for _, curRhs := range s.Rhs {
		rlist, ok := proof.Results[curRhs.Secret]
		if !ok {
			return false
		}
		if len(rlist) != rangeProofIters {
			return false
		}
		for _, val := range rlist {
			if val == nil {
				return false
			}
		}
	}

	// Validate size of secret results
	rangeLimit := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon+2)
	for _, val := range proof.Results[s.rangeSecret] {
		if val.Cmp(rangeLimit) >= 0 {
			return false
		}
	}

	return true
}

type rangeProofResultLookup struct {
	Results map[string]*big.Int
}

func (r *rangeProofResultLookup) ProofResult(name string) *big.Int {
	res, ok := r.Results[name]
	if !ok {
		return nil
	}
	return res
}

func (s *rangeProofStructure) commitmentsFromProof(g zkproof.Group, list []*big.Int, challenge *big.Int, bases zkproof.BaseLookup, proof RangeProof) []*big.Int {
	// Some values needed in all iterations
	resultOffset := new(big.Int).Lsh(big.NewInt(1), s.l2+rangeProofEpsilon+1)
	l1Offset := new(big.Int).Lsh(big.NewInt(1), s.l1)

	// Iterate over all indices
	for i := 0; i < rangeProofIters; i++ {
		// Build resultLookup
		resultLookup := rangeProofResultLookup{map[string]*big.Int{}}
		for name, rlist := range proof.Results {
			var res *big.Int
			if name == s.rangeSecret {
				res = new(big.Int).Sub(rlist[i], resultOffset)
				if challenge.Bit(i) == 1 {
					res.Sub(res, l1Offset)
				}
			} else {
				res = new(big.Int).Set(rlist[i])
			}
			resultLookup.Results[name] = res
		}

		// And generate commitment
		list = s.RepresentationProofStructure.CommitmentsFromProof(g, list, big.NewInt(int64(challenge.Bit(i))), bases, &resultLookup)
	}

	Follower.Tick()

	return list
}

func (r *rangeCommitSecretLookup) Secret(_ string) *big.Int {
	return nil
}

func (r *rangeCommitSecretLookup) Randomizer(name string) *big.Int {
	clist, ok := r.commits[name]
	if !ok {
		return nil
	}
	return clist[r.i]
}

func (s *rangeProofStructure) numRangeProofs() int {
	return 1
}

func (s *rangeProofStructure) numCommitments() int {
	return rangeProofIters
}
