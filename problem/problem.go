package problem

type Problem interface {
	GetContestName()
	GetProblemName()
	GetTimeLimit()
	GetMemoryLimit()
	GetSampleInputs()
	GetSampleOutpus()
}
