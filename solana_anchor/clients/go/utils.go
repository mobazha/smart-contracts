package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// 展开路径中的 ~ 为用户主目录
func expandPath(path string) string {
	if strings.HasPrefix(path, "~/") {
		home, err := os.UserHomeDir()
		if err != nil {
			return path
		}
		return filepath.Join(home, path[2:])
	}
	return path
}

// 将 SOL 转换为 lamports
func SolToLamports(sol float64) uint64 {
	return uint64(sol * 1e9)
}

// 将 lamports 转换为 SOL
func LamportsToSol(lamports uint64) float64 {
	return float64(lamports) / 1e9
}

// 格式化 SOL 金额
func FormatSol(lamports uint64) string {
	return fmt.Sprintf("%.9f SOL", LamportsToSol(lamports))
}
