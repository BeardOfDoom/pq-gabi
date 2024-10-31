package gabi

import (
	"bytes"
	"crypto/sha256"

	"github.com/cbergoon/merkletree"
)

// Attribute implements the Content interface provided by merkletree and represents the content stored in the tree.
type Attribute struct {
	value []byte
}

// CalculateHash hashes the values of a Attribute
func (t Attribute) CalculateHash() ([]byte, error) {
	h := sha256.New()
	if _, err := h.Write([]byte(t.value)); err != nil {
		return nil, err
	}

	return h.Sum(nil), nil
}

// Equals tests for equality of two Attributes
func (t Attribute) Equals(other merkletree.Content) (bool, error) {
	return bytes.Equal(t.value, other.(Attribute).value), nil
}
