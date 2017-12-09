package problem

type Problem interface {
	GetContestName()
	GetProblemName()
	GetSampleInputs()
	GetSampleOutpus()
}
