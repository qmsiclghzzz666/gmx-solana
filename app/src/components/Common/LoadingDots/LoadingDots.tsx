import { GoDotFill } from 'react-icons/go';
import './LoadingDots.scss';

const LoadingDots = ({ size = 14 }: { size?: number }) => {
  return (
    <div className="loading-dots">
      <GoDotFill fontSize={size} />
      <GoDotFill fontSize={size} />
      <GoDotFill fontSize={size} />
    </div>
  );
};

export default LoadingDots;
