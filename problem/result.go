package problem

type Result struct {
	StdOutput string
	TestCase  TestCase
	ExecTime  float32
}

func (r *Result) IsAccept() bool {
	if r.StdOutput == r.TestCase.Output {
		return true
	}
	return false
}

func (r *Result) IsWrongAnswer() bool {
	if r.StdOutput != r.TestCase.Output {
		return true
	}
	return false
}
